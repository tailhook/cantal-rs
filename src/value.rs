use std::fmt::Display;

use libc::c_void;
use serde_json;

pub enum LevelKind {
    Signed,
    Float,
}

pub enum RawType {
    Level(LevelKind),
    Counter,
    State,
}

pub trait Value: Display + Assign {
    fn raw_type(&self) -> RawType;
    fn raw_size(&self) -> usize;
    fn as_json(&self) -> serde_json::Value;
}

pub trait Assign {
    fn assign(&self, ptr: *mut c_void);
    fn reset(&self);
}

impl RawType {
    pub fn as_json_str(&self) -> &'static str {
        use self::RawType::*;

        match *self {
            Level(_) => "level",
            Counter => "counter",
            State => "state",
        }
    }
    pub fn main_type(&self) -> &'static str {
        use self::RawType::*;

        match *self {
            Level(_) => "level",
            Counter => "counter",
            State => "state",
        }
    }
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
