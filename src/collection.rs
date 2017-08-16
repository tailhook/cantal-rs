use std::env;
use std::fs::{OpenOptions, remove_file, rename};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::ptr;

use libc;
use serde_json::{to_string, to_value};

use error::{Error, ErrorEnum};
use name::Name;
use value::{Value, RawType};
use json::JsonName;
use {ActiveCollection};


/// A trait used to enumerate a collection
pub trait Visitor<'a> {
    /// Report a metric that belongs to a collection
    fn metric(&mut self, name: &Name, value: &'a Value);
}

/// A collection of metrics
///
/// A single collection is usually exported by a process but a vector or
/// slice of collections is also a collections, so they can be combined easily.
pub trait Collection {
    /// Visit the whole collection, use visitor to report a metric
    fn visit<'x>(&'x self, visitor: &mut Visitor<'x>);
}

#[cfg(unix)]
fn configured_path(warn: bool) -> Option<(PathBuf, String)> {
    env::var_os("CANTAL_PATH").and_then(|path| {
        let path = Path::new(&path);
        path.parent()
        .and_then(|p| path.file_name()
            .and_then(|x| x.to_str())
            .map(|n| (p.to_path_buf(), n.to_string())))
        .or_else(|| {
            if warn {
                error!("CANTAL_PATH is present, but can't be split into \
                    a directory and a filename. \
                    Probably contains som non-utf-8 characters.");
            }
            None
        })
    })
}

#[cfg(unix)]
pub fn path_from_env(warn: bool) -> (PathBuf, String) {
    use libc::{getpid, getuid};

    if let Some((dir, name)) = configured_path(warn) {
        (dir, name)
    } else {
        let (dir, name) = if let Some(dir) = env::var_os("XDG_RUNTIME_DIR") {
            (PathBuf::from(&dir), format!("cantal.{}", unsafe { getpid() }))
        } else {
            (PathBuf::from("/tmp"),
             format!("cantal.{}.{}", unsafe { getuid() }, unsafe { getpid() }))
        };
        if warn {
            warn!(
                "No CANTAL_PATH is set in the environment, using {:?}. \
                 The cantal-agent will be unable to discover it.",
                dir.join(&name));
        }
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
pub fn start<'x, T: Collection + ?Sized>(coll: &'x T)
    -> Result<ActiveCollection<'x>, Error>
{
    use std::os::unix::io::AsRawFd;
    use self::ErrorEnum::*;

    struct Metric<'a> {
        name: String,
        raw_type: RawType,
        size: usize,
        pointer: &'a Value,
    }
    struct ListVisitor<'a, 'b: 'a, 'c: 'a>(&'a mut Vec<Metric<'b>>,
        &'a mut Vec<&'c Value>, &'a mut usize);
    impl<'a, 'b: 'a, 'c: 'a> Visitor<'b> for ListVisitor<'a, 'b, 'c> {
        fn metric(&mut self, name: &Name, value: &'b Value)
        {
            self.0.push(Metric {
                // must have all keys sorted
                name: to_string(&to_value(JsonName(name))
                    .expect("can always serialize"))
                    .expect("can always serialize"),
                raw_type: value.raw_type(),
                size: value.raw_size(),
                pointer: value,
            });
            *self.2 += value.raw_size();
        }
    }

    let mut values_size = 0;
    let mut pointers = Vec::new();
    let mut all_metrics = Vec::with_capacity(100);
    coll.visit(&mut ListVisitor(&mut all_metrics, &mut pointers,
                                &mut values_size));

    // TODO(tailhook) sort metrics by size class
    // TODO(tailhook) find out real page size
    values_size = (values_size + 4095) & !4095;

    let (dir, name) = path_from_env(true);
    let tmp_path = dir.join(format!("{}.tmp", name));
    let values_path = dir.join(format!("{}.values", name));
    let meta_path = dir.join(format!("{}.meta", name));
    remove_if_exists(&tmp_path)?;
    remove_if_exists(&values_path)?;
    remove_if_exists(&meta_path)?;

    let values_file = OpenOptions::new()
        .read(true).write(true).create_new(true)
        .open(&tmp_path)
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
    if ptr == libc::MAP_FAILED {
        let err = io::Error::last_os_error();
        remove_file(&tmp_path).map_err(|e| {
            error!("Can't unlink path {:?}: {}", tmp_path, e);
        }).ok();
        return Err(Mmap(tmp_path, err, values_size).into());
    }

    // Create our unmap/reset guard before setting actual pointers
    let result = ActiveCollection {
        meta_path: meta_path,
        values_path: values_path,
        metrics: pointers,
        mmap: ptr,
        mmap_size: values_size,
    };

    let mut offset = 0;
    let mut metadata_buf = String::with_capacity(4096);
    let mut pointers = Vec::new();
    for metric in all_metrics {
        use std::fmt::Write;
        write!(metadata_buf, "{main_type} {size}{space}{type_suffix}: {key}\n",
            main_type=metric.raw_type.main_type(),
            size=metric.size,
            space=if metric.raw_type.type_suffix().is_some() { " " } else {""},
            type_suffix=metric.raw_type.type_suffix().unwrap_or(""),
            key=metric.name)
            .expect("Can always write into buffer");
        metric.pointer.copy_assign(unsafe { ptr.offset(offset as isize) });
        offset += metric.size;
        pointers.push(metric.pointer);
    }

    rename(&tmp_path, &result.values_path)
        .map_err(|e| Rename(result.values_path.clone(), e))?;
    OpenOptions::new().write(true).create_new(true)
        .open(&tmp_path)
        .and_then(|mut f| f.write_all(metadata_buf.as_bytes()))
        .map_err(|e| WriteMetadata(tmp_path.clone(), e))?;
    rename(&tmp_path, &result.meta_path)
        .map_err(|e| Rename(result.meta_path.clone(), e))?;

    Ok(result)
}

/// Start publishing metrics
///
/// Currently it's noop on windows
#[cfg(windows)]
pub fn start<'x, T: Collection + ?Sized>(coll: &'x T)
    -> Result<ActiveCollection<'x>, Error>
{
    // TODO(tailhook) maybe always put into temporary directory?
    ActiveCollection {
        phantom: ::std::marker::PhantomData,
    }
}

#[cfg(unix)]
impl<'a> Drop for ActiveCollection<'a> {
    fn drop(&mut self) {
        for m in &self.metrics {
            m.reset();
        }
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
