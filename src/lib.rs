//! A library that exposes metrics sent to cantal
//!
//! See [mmap protocol][proto] for more information about the protocol.
//!
//! [proto]: http://cantal.readthedocs.io/en/latest/mmap.html
//!
//! # Example
//!
//! ```rust
//! # extern crate libcantal;
//! # #[macro_use] extern crate lazy_static;
//! # #[macro_use] extern crate serde_json;
//! # use std::io::stdout;
//! use libcantal::{Counter, Integer, Value, start, print};
//!
//! // Declare metrics as static variables, so you can use it in a module
//! // freely, i.e. increment/decrement at any time
//! lazy_static! {
//!     static ref COUNTER: Counter = Counter::new();
//!     static ref INTEGER: Integer = Integer::new();
//! }
//!
//! # fn main() {
//! // Put metrics in a collection
//! let metrics = [
//!     (json!({"metric": "counter"}), &*COUNTER as &Value),
//!     (json!({"metric": "integer"}), &*INTEGER as &Value),
//! ];
//! // The collection guard. When it's alive, all the metrics are exposed
//! let _coll = start(&metrics[..]).expect("cantal works");
//!
//! // ...
//! // Somewhere later use the metrics:
//! COUNTER.incr(1);
//! INTEGER.set((COUNTER.get() / 7) as i64);
//!
//! // And at any time you can print the collection for debugging
//! print(&metrics[..], stdout()).ok();
//! #
//! # }
//! ```
#![warn(missing_docs)]
extern crate atomic;
extern crate libc;
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;

mod collection;
mod collections;
mod error;
mod json;
mod name;
mod names;
mod print;
mod read;
mod value;

mod counter;
mod integer;

pub use collection::{Collection, Visitor, start};
pub use counter::Counter;
pub use error::Error;
pub use integer::Integer;
pub use json::Json;
pub use name::{NameVisitor, Name};
pub use print::print;
pub use read::{start_with_reading};
pub use value::{Value, RawType, LevelKind};

use std::path::PathBuf;

/// An active collection currently publishing metrics
///
/// It's basically a guard: if you drop it, metrics are not exported any more.
#[cfg(unix)]
pub struct ActiveCollection<'a> {
    values_path: PathBuf,
    meta_path: PathBuf,
    metrics: Vec<&'a Value>,
    mmap: *mut libc::c_void,
    mmap_size: usize,
}

/// An active collection currently publishing metrics
///
/// It's basically a guard: if you drop it, metrics are not exported any more.
///
/// Note: not implemented for windows yet.
#[cfg(windows)]
pub struct ActiveCollection<'a> {
    phantom: ::std::marker::PhantomData<&'a ()>
}
