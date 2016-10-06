use std::fmt;
use std::mem::transmute;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};

pub struct Counter {
    #[allow(dead_code)]
    value: Box<AtomicUsize>,
    pointer: AtomicPtr<AtomicUsize>,
}

impl Counter {
    pub fn new() -> Counter {
        let tmp = Box::new(AtomicUsize::new(0));
        Counter {
            pointer: unsafe { transmute(&*tmp) },
            value: tmp,
        }
    }
    pub fn incr(&self, val: usize) {
        unsafe {
            &*self.pointer.load(Ordering::Relaxed)
        }.fetch_add(val, Ordering::Relaxed);
    }
    pub fn get(&self) -> usize {
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
