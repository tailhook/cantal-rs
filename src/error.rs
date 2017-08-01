use std::io;
use std::path::PathBuf;


quick_error! {
    /// Error of exporting metrics
    ///
    /// Can be triggered on ``cantal::start`` or ``cantal::start_by_reading``.
    /// And may be ignored if metrics are not crucial for the application.
    #[derive(Debug)]
    pub enum Error wraps pub ErrorEnum {
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
        Rename(path: PathBuf, err: io::Error) {
            display("Can't rename file to {:?}: {}", path, err)
            description("Can't rename file")
            cause(err)
        }
        WriteMetadata(path: PathBuf, err: io::Error) {
            display("Can't write metadata to {:?}: {}", path, err)
            description("Can't write metadata")
            cause(err)
        }
        Read(path: PathBuf, err: io::Error) {
            display("IO error {:?}: {}", path, err)
            description("IO error")
            cause(err)
        }
        InvalidMeta(path: PathBuf, err: &'static str) {
            display("error parsing metadata file {:?}: {}", path, err)
            description(err)
        }
    }
}
