use std::collections::HashMap;
use std::ptr;
use std::io::{self, BufRead, BufReader};
use std::fs::{File, OpenOptions};

use libc;
use serde_json::{to_string, to_value};
use json::JsonName;

use collection::{start, path_from_env};

use error::{Error, ErrorEnum};
use {Name, Value, Visitor, Collection, ActiveCollection};

/// Start publishing metrics
#[cfg(windows)]
pub fn start_with_reading<'x, T: Collection + ?Sized>(coll: &'x T)
    -> Result<ActiveCollection<'x>, Error>
{
    start(coll)
}

/// Start publishing metrics by reading old values first
///
/// Note: usually you don't need this method and just use ``start``. This
/// constructor is only useful if you have really fast restarting service
/// (i.e. it often restart faster than cantal's scan interval,
/// which is 2 seconds). Also if your program restarts normally, it will
/// clean file with metrics on exit. This method is originally used in
/// process that restarts in-place by using ``execve`` so destructors don't
/// run. This constructor is also slightly more expensive.
///
/// # Concurrent Use
///
/// If used very carefully this method can also be used to keep
/// several processes writing to the same file with metrics but this use is
/// not thoroughtly tested and very limited, known limitations:
///
/// 1. Same metrics must be used by all processes (file will be overwritten
///    by a random process if not)
/// 2. `start_with_reading` should not be run concurrently (some external
///    locking is required)
/// 3. Levels (gauges) should either be externally synchronized or adjusted
///    by `incr()/decr()` instructions (not `set()`)
/// 4. Process crash may leave some counters / gauges non-adjusted
///
#[cfg(unix)]
pub fn start_with_reading<'x, T: Collection + ?Sized>(coll: &'x T)
    -> Result<ActiveCollection<'x>, Error>
{
    match read_and_map(coll) {
        Ok(Some(x)) => Ok(x),
        Ok(None) => start(coll),
        Err(e) => {
            warn!("Error reading old metrics: {}. \
                Trying to create new files...", e);
            start(coll)
        }
    }
}

pub fn read_and_map<'x, T: Collection + ?Sized>(coll: &'x T)
    -> Result<Option<ActiveCollection<'x>>, Error>
{
    use std::os::unix::io::AsRawFd;
    let (dir, name) = path_from_env(false); // will warn in start
    let values_path = dir.join(format!("{}.values", name));
    let meta_path = dir.join(format!("{}.meta", name));

    let meta_file = match File::open(&meta_path) {
        Ok(file) => file,
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(ErrorEnum::Read(meta_path, e).into());
            }
            return Ok(None);
        }
    };
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(false).truncate(false);
    let values_file = match options.open(&values_path) {
        Ok(file) => file,
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(ErrorEnum::Read(values_path, e).into());
            }
            return Ok(None);
        }
    };
    let values_size = values_file.metadata()
        .map_err(|e| ErrorEnum::Read(meta_path.clone(), e))?
        .len() as usize;

    let ptr = unsafe {
        libc::mmap(ptr::null_mut(), values_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            values_file.as_raw_fd(),
            0)
    };
    if ptr == libc::MAP_FAILED {
        let err = io::Error::last_os_error();
        return Err(ErrorEnum::Mmap(values_path, err, values_size).into());
    }

    // Create our unmap/reset guard before setting actual pointers
    let mut result = ActiveCollection {
        meta_path: meta_path.clone(),
        values_path: values_path,
        metrics: Vec::new(),
        mmap: ptr,
        mmap_size: values_size,
    };


    struct MapVisitor<'a, 'b: 'a>(&'a mut HashMap<String, &'b Value>);
    impl<'a, 'b: 'a> Visitor<'b> for MapVisitor<'a, 'b> {
        fn metric(&mut self, name: &Name, value: &'b Value)
        {
            // we encode to_value first to get keys sorted
            self.0.insert(to_string(&to_value(JsonName(name))
                    .expect("can always serialize"))
                    .expect("can always serialize"),
                    value);
        }
    }

    let mut map = HashMap::new();
    coll.visit(&mut MapVisitor(&mut map));

    let mut extra = 0;
    let mut wrong_type = 0;

    let mut offset = 0;
    for line in BufReader::new(meta_file).lines() {
        let line = line.map_err(|e| ErrorEnum::Read(meta_path.clone(), e))?;
        let mut pair = line.splitn(2, ":");
        let mut type_iter = pair.next().unwrap().split(' ');
        let kind = type_iter.next().unwrap();
        let size = type_iter.next()
            .ok_or_else(|| ErrorEnum::InvalidMeta(
                meta_path.clone(), "Unsized type"))?
            .parse().map_err(|_| ErrorEnum::InvalidMeta(
                meta_path.clone(), "Can't parse type size"))?;
        let suffix = type_iter.next();
        if (offset + size) as usize > values_size {
            return Err(ErrorEnum::InvalidMeta(
                meta_path.clone(), "Offset is out of range").into());
        }
        if kind == "pad" {
            offset += size;
            continue;
        }
        let name = pair.next()
            .ok_or_else(|| ErrorEnum::InvalidMeta(meta_path.clone(),
                                              "No description for value"))?
            .trim();
        if let Some(metric) = map.remove(name) {
            let typ = metric.raw_type();
            if kind != typ.main_type() || suffix != typ.type_suffix() {
                wrong_type += 1;
            } else {
                unsafe {
                    metric.assign(ptr.offset(offset));
                    result.metrics.push(metric);
                }
            }
        } else {
            extra += 1;
        }
        offset += size;
    }
    if extra > 0 || map.len() > 0 {
        debug!("Found {} extra metrics, {} metrics are not present, \
                {} have different type. \
                Copying metrics and overriding file.",
               extra, map.len(), wrong_type);
        return Ok(None)
    } else {
        debug!("Continuing with {} metrics and {}/{} bytes",
            result.metrics.len(), offset, values_size);
    }
    Ok(Some(result))
}
