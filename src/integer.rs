use std::fmt;
use std::mem::transmute;
use std::sync::atomic::{AtomicPtr, Ordering};

use atomic::Atomic;
use libc::c_void;
use serde_json;

use value::{Value, RawType, LevelKind, Assign};


/// A kind of metric (`Value`) that exports gauge with integer value
pub struct Integer {
    #[allow(dead_code)]
    value: Box<Atomic<i64>>,
    pointer: AtomicPtr<Atomic<i64>>,
}

impl Integer {
    /// Create a new integer gauge value
    ///
    /// Note you need to export it in a collection to make it visible for
    /// cantal agent
    pub fn new() -> Integer {
        let tmp = Box::new(Atomic::new(0));
        Integer {
            pointer: unsafe { transmute(&*tmp) },
            value: tmp,
        }
    }
    /// Increase the value of a gauge
    pub fn incr(&self, val: i64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_add(val, Ordering::Relaxed);
    }
    /// Decrease the value of a gauge
    pub fn decr(&self, val: i64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_sub(val, Ordering::Relaxed);
    }
    /// Set (replace) the value of a gauge
    pub fn set(&self, val: i64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.store(val, Ordering::Relaxed);
    }
    /// Get current value of a gauge
    ///
    /// Note it works regardless of whether it's attached to a value
    pub fn get(&self) -> i64 {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.load(Ordering::Relaxed)
    }
}


impl fmt::Display for Integer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.load(Ordering::Relaxed))
    }
}

impl Value for Integer {
    fn raw_type(&self) -> RawType { RawType::Level(LevelKind::Signed) }
    fn raw_size(&self) -> usize { 8 }
    fn as_json(&self) -> serde_json::Value {
        serde_json::Value::Number(self.get().into())
    }
}

impl Assign for Integer {
    fn assign(&self, ptr: *mut c_void) {
        self.pointer.store(unsafe { transmute(ptr) }, Ordering::SeqCst);
    }
    fn reset(&self) {
        self.pointer.store(unsafe { transmute(&*self.value) },
                           Ordering::SeqCst);
    }
}
