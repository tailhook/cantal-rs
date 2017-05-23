use std::fmt;
use std::mem::transmute;
use std::sync::atomic::{AtomicPtr, Ordering};

use atomic::Atomic;
use serde_json;

use value::{Value, RawType};


pub struct Counter {
    #[allow(dead_code)]
    value: Box<Atomic<u64>>,
    pointer: AtomicPtr<Atomic<u64>>,
}

impl Counter {
    pub fn new() -> Counter {
        let tmp = Box::new(Atomic::new(0));
        Counter {
            pointer: unsafe { transmute(&*tmp) },
            value: tmp,
        }
    }
    pub fn incr(&self, val: u64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_add(val, Ordering::Relaxed);
    }
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
