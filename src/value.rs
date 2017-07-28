use std::fmt::Display;

use libc::c_void;
use serde_json;


/// A kind of level (gauge) metric, only used for `Value` trait
pub enum LevelKind {
    /// Signed integer gauge type
    Signed,
    /// Floating point gauge type
    Float,
}

/// A raw type of metric, only used for `Value` trait
pub enum RawType {
    /// Level or gauge type of metric
    Level(LevelKind),
    /// An ever-increasing counter
    Counter,
    /// Just a string value that can be exposed to the collection system
    State,
}

/// A value stored in a collection
///
/// Usually you don't implement values yourself, as only selected kinds of
/// metrics are supported by cantal agent, and all of them are implemented in
/// this crate itself
pub trait Value: Display + Assign {
    /// Returns raw type (type as stored in the file)
    fn raw_type(&self) -> RawType;
    /// Returns the size in bytes for the type
    fn raw_size(&self) -> usize;
    /// Returns JSONified value of a metric
    fn as_json(&self) -> serde_json::Value;
}

pub trait Assign {
    fn assign(&self, ptr: *mut c_void);
    fn reset(&self);
}

impl RawType {
    /// Returns JSON-friendly type of a value
    pub fn as_json_str(&self) -> &'static str {
        use self::RawType::*;

        match *self {
            Level(_) => "level",
            Counter => "counter",
            State => "state",
        }
    }
    /// Returns main type of the value (specifics for cantal format)
    ///
    /// See [memory map protocol][proto] for more info
    ///
    /// [proto]: http://cantal.readthedocs.io/en/latest/mmap.html
    pub fn main_type(&self) -> &'static str {
        use self::RawType::*;

        match *self {
            Level(_) => "level",
            Counter => "counter",
            State => "state",
        }
    }
    /// Returns type suffix of the value (specifics for cantal format)
    ///
    /// See [memory map protocol][proto] for more info
    ///
    /// [proto]: http://cantal.readthedocs.io/en/latest/mmap.html
    pub fn type_suffix(&self) -> Option<&'static str> {
        use self::RawType::*;
        use self::LevelKind::*;

        match *self {
            Level(Signed) => Some("signed"),
            Level(Float) => Some("float"),
            Counter => None,
            State => None,
        }
    }
}
