use std::env;
use std::path::{Path, PathBuf};

use name::Name;
use value::Value;


pub trait Visitor {
    fn metric<N: Name, V: Value+?Sized>(&mut self, name: &N, value: &V);
}

pub trait Collection {
    fn visit<V: Visitor>(&self, visitor: &mut V);
}

/// Start publishing metrics
#[cfg(unix)]
pub fn start<T: Collection>(coll: T) -> PathBuf {
    use libc::{getpid, getuid};

    let path = if let Some(path) = env::var_os("CANTAL_PATH") {
        PathBuf::from(path)
    } else {
        let path = if let Some(dir) = env::var_os("XDG_RUNTIME_DIR") {
            Path::new(&dir).join(format!("cantal.{}", unsafe { getpid() }))
        } else {
            PathBuf::from(format!("/tmp/cantal.{}.{}",
                unsafe { getuid() }, unsafe { getpid() }))
        };
        warn!(
            "No CANTAL_PATH is set in the environment, using {:?}. \
             The cantal-agent will be unable to discover it.",
            path);
        path
    };
    unimplemented!();
    path
}

/// Removes published metrics
#[cfg(unix)]
pub fn cleanup(path: &Path) {
    unimplemented!();
}

/// Executes a function and then does cleanup of metrics
///
/// Just a shortcut for:
///
/// ```rust,ignore
///
/// let path = start(coll);
/// f();
/// cleanup(&path);
/// ```
pub fn context<T: Collection, F, R>(coll: T, f: F) -> R
    where F: FnOnce() -> R
{
    let path = start(coll);
    let result = f();
    cleanup(&path);
    result
}

/// Start publishing metrics
///
/// Currently it's noop on windows
#[cfg(windows)]
pub fn start<T: Collection>(_coll: Collection) {
    // TODO(tailhook) maybe always put into temporary directory?
}

/// Removes published metrics
///
/// Currently it's noop on windows
#[cfg(windows)]
pub fn cleanup(_path: &Path) {
    // TODO(tailhook) maybe always put into temporary directory?
}
