extern crate libcantal;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;

use std::io::stdout;
use std::time::Duration;
use std::thread::sleep;

use serde_json::to_writer_pretty;

use libcantal::{Counter, Integer, Value, Json};


lazy_static! {
    static ref COUNTER: Counter = Counter::new();
    static ref INTEGER: Integer = Integer::new();
}

fn main() {
    loop {
        COUNTER.incr(1);
        INTEGER.set((COUNTER.get() / 7) as i64);
        to_writer_pretty(stdout(), &Json(&[
            (json!({"metric": "counter"}), &*COUNTER as &Value),
            (json!({"metric": "integer"}), &*INTEGER as &Value),
        ][..])).expect("printing should work");
        sleep(Duration::new(1, 0));
    }
}
