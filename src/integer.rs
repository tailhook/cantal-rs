use std::fmt;
use std::mem::transmute;
use std::sync::atomic::{AtomicPtr, Ordering};

use atomic::Atomic;

use value::Value;


pub struct Integer {
    #[allow(dead_code)]
    value: Box<Atomic<i64>>,
    pointer: AtomicPtr<Atomic<i64>>,
}

impl Integer {
    pub fn new() -> Integer {
        let tmp = Box::new(Atomic::new(0));
        Integer {
            pointer: unsafe { transmute(&*tmp) },
            value: tmp,
        }
    }
    pub fn incr(&self, val: i64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_add(val, Ordering::Relaxed);
    }
    pub fn decr(&self, val: i64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_sub(val, Ordering::Relaxed);
    }
    pub fn set(&self, val: i64) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.store(val, Ordering::Relaxed);
    }
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
}
