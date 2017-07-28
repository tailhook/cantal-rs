use std::fmt;
use std::mem::transmute;
use std::sync::atomic::{AtomicPtr, Ordering};

use atomic::Atomic;
use libc::c_void;
use serde_json;

use value::{Value, RawType, Assign};


/// A kind of metric (`Value`) that exports ever-increasing counter
pub struct Counter {
    #[allow(dead_code)]
    value: Box<Atomic<u64>>,
    pointer: AtomicPtr<Atomic<u64>>,
}

impl Counter {
    /// Create a new counter
    ///
    /// Note you need to export it in a collection to make it visible for
    /// cantal agent
    pub fn new() -> Counter {
        let tmp = Box::new(Atomic::new(0));
        Counter {
            pointer: unsafe { transmute(&*tmp) },
            value: tmp,
        }
    }
    /// Increase a counter for ``val``
    pub fn incr(&self, val: u64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_add(val, Ordering::Relaxed);
    }
    /// Get current value for counter
    ///
    /// Note it works regardless of whether it's attached to a value
    pub fn get(&self) -> u64 {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.load(Ordering::Relaxed)
    }
}


impl fmt::Display for Counter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.load(Ordering::Relaxed))
    }
}

impl Value for Counter {
    fn raw_type(&self) -> RawType { RawType::Counter }
    fn raw_size(&self) -> usize { 8 }
    fn as_json(&self) -> serde_json::Value {
        serde_json::Value::Number(self.get().into())
    }
}

impl Assign for Counter {
    fn assign(&self, ptr: *mut c_void) {
        self.pointer.store(unsafe { transmute(ptr) }, Ordering::SeqCst);
    }
    fn reset(&self) {
        self.pointer.store(unsafe { transmute(&*self.value) },
                           Ordering::SeqCst);
    }
}
