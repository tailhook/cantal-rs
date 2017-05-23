use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs::{File, remove_file, rename};
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::ptr;

use libc;
use serde_json::{to_string, to_value};

use name::Name;
use value::{Value, RawType};
use json::JsonName;


pub trait Visitor {
    fn metric(&mut self, name: &Name, value: &Value);
}

pub trait Collection {
    fn visit(&self, visitor: &mut Visitor);
}

#[cfg(unix)]
pub struct ActiveCollection {
    values_path: PathBuf,
    meta_path: PathBuf,
    mmap: *mut libc::c_void,
    mmap_size: usize,
}

#[cfg(windows)]
pub struct ActiveCollection {
}

quick_error! {
    #[derive(Debug)]
    pub enum Error wraps ErrorEnum {
        Delete(path: PathBuf, err: io::Error) {
            display("Can't delete file {:?}: {}", path, err)
            description("Can't delete file")
            cause(err)
        }
        Create(path: PathBuf, err: io::Error) {
            display("Can't create file {:?}: {}", path, err)
            description("Can't create file")
            cause(err)
        }
        Mmap(path: PathBuf, err: io::Error, size: usize) {
            display("Can't mmap file {:?} size {}: {}", path, size, err)
            description("Can't mmap file")
            cause(err)
        }
    }
}

impl<'a> Collection for &'a Collection {
    fn visit(&self, visitor: &mut Visitor) {
        (*self).visit(visitor)
    }
}
impl<'a, T: Collection + 'a> Collection for &'a T {
    fn visit(&self, visitor: &mut Visitor) {
        (*self).visit(visitor)
    }
}

#[cfg(unix)]
fn configured_path() -> Option<(PathBuf, String)> {
    env::var_os("CANTAL_PATH").and_then(|path| {
        let path = Path::new(&path);
        path.parent()
        .and_then(|p| path.file_name()
            .and_then(|x| x.to_str())
            .map(|n| (p.to_path_buf(), n.to_string())))
        .or_else(|| {
            error!("CANTAL_PATH is present, but can't be split into \
                a directory and a filename. \
                Probably contains som non-utf-8 characters.");
            None
        })
    })
}

#[cfg(unix)]
fn path_from_env() -> (PathBuf, String) {
    use libc::{getpid, getuid};

    if let Some((dir, name)) = configured_path() {
        (dir, name)
    } else {
        let (dir, name) = if let Some(dir) = env::var_os("XDG_RUNTIME_DIR") {
            (PathBuf::from(&dir), format!("cantal.{}", unsafe { getpid() }))
        } else {
            (PathBuf::from("/tmp"),
             format!("cantal.{}.{}", unsafe { getuid() }, unsafe { getpid() }))
        };
        warn!(
            "No CANTAL_PATH is set in the environment, using {:?}. \
             The cantal-agent will be unable to discover it.",
            dir.join(&name));
        (dir, name)
    }
}

fn remove_if_exists(path: &Path) -> Result<(), Error> {
    use self::ErrorEnum::*;

    match remove_file(path) {
        Ok(()) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Delete(path.to_path_buf(), e).into()),
    }
}

/// Start publishing metrics
#[cfg(unix)]
pub fn start<T: Collection>(coll: T) -> Result<ActiveCollection, Error> {
    use self::ErrorEnum::*;

    struct Metric {
        name: String,
        raw_type: RawType,
        size: usize,
    }
    struct ListVisitor<'a>(&'a mut Vec<Metric>);
    impl<'a> Visitor for ListVisitor<'a> {
        fn metric(&mut self, name: &Name, value: &Value)
        {
            self.0.push(Metric {
                // must have all keys sorted
                name: to_string(&to_value(JsonName(name))
                    .expect("can always serialize"))
                    .expect("can always serialize"),
                raw_type: value.raw_type(),
                size: value.raw_size(),
            })
        }
    }

    let mut all_metrics = Vec::with_capacity(100);
    coll.visit(&mut ListVisitor(&mut all_metrics));

    // TODO(tailhook) sort metrics by size class

    let mut offset = 0;
    let mut metadata_buf = String::with_capacity(4096);
    let mut offsets = HashMap::new();
    for metric in &all_metrics {
        // must have all keys sorted
        write!(metadata_buf, "{main_type} {size}{space}{type_suffix}: {key}",
            main_type=metric.raw_type.main_type(),
            size=metric.size,
            space=if metric.raw_type.type_suffix().is_some() { " " } else {""},
            type_suffix=metric.raw_type.type_suffix().unwrap_or(""),
            key=metric.name)
        .expect("Can always write into buffer");
        offsets.insert(&metric.name, offset);
        offset += metric.size;
    }
    // TODO(tailhook) find out real page size
    let values_size = offset + offset & 4095;

    let (dir, name) = path_from_env();
    let tmp_path = dir.join(format!("{}.tmp", name));
    let values_path = dir.join(format!("{}.values", name));
    let meta_path = dir.join(format!("{}.meta", name));
    remove_if_exists(&tmp_path)?;
    remove_if_exists(&values_path)?;
    remove_if_exists(&meta_path)?;

    let values_file = File::create(&tmp_path)
        .map_err(|e| Create(tmp_path.clone(), e))?;
    values_file.set_len(values_size as u64)
        .map_err(|e| Create(tmp_path.clone(), e))?;
    let ptr = unsafe {
        libc::mmap(ptr::null_mut(), values_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            values_file.as_raw_fd(),
            0)
    };
    if ptr == ptr::null_mut() {
        let err = io::Error::last_os_error();
        remove_file(&tmp_path).map_err(|e| {
            error!("Can't unlink path {:?}: {}", tmp_path, e);
        }).ok();
        return Err(Mmap(tmp_path, err, values_size).into());
    }

    let mut current_values = Vec::new();

    struct AssignVisitor<'a>(HashMap<&'a String, usize>, *mut libc::c_void);
    impl<'a> Visitor for AssignVisitor<'a> {
        fn metric(&mut self, name: &Name, value: &Value)
        {
            let key =
                // must have all keys sorted
                to_string(&to_value(JsonName(name))
                    .expect("can always serialize"))
                    .expect("can always serialize");
            let off = self.0.get(key)
                .expect("collection has not changed during assignment");
            assign(value, self.1 + off);
        }
    }

    coll.visit(&mut AssignVisitor(offsets, ptr));

    let result = rename(&tmp_path, &values_path);

    Ok(ActiveCollection {
        meta_path: meta_path,
        values_path: values_path,
        mmap: ptr,
        mmap_size: values_size,
    })
}

/// Start publishing metrics
///
/// Currently it's noop on windows
#[cfg(windows)]
pub fn start<T: Collection>(_coll: Collection)
    -> Result<ActiveCollection, Error>
{
    // TODO(tailhook) maybe always put into temporary directory?
    ActiveCollection {}
}

#[cfg(unix)]
impl Drop for ActiveCollection {
    fn drop(&mut self) {
        let rc = unsafe { libc::munmap(self.mmap, self.mmap_size) };
        if rc != 0 {
            let err = io::Error::last_os_error();
            error!("Can't unmap file {:?}: {}", self.values_path, err);
        }
        remove_file(&self.values_path).map_err(|e| {
            error!("Can't unlink path {:?}: {}", self.values_path, e);
        }).ok();
        remove_file(&self.meta_path).map_err(|e| {
            error!("Can't unlink path {:?}: {}", self.meta_path, e);
        }).ok();
    }
}
