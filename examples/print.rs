extern crate libcantal;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;

use std::io::stdout;
use std::time::Duration;
use std::thread::sleep;

use libcantal::{Counter, Integer, print, Value};


lazy_static! {
    static ref COUNTER: Counter = Counter::new();
    static ref INTEGER: Integer = Integer::new();
}

fn main() {
    loop {
        COUNTER.incr(1);
        INTEGER.set((COUNTER.get() / 7) as i64);
        print(&[
            (json!({"metric": "counter"}), &*COUNTER as &Value),
            (json!({"metric": "integer"}), &*INTEGER as &Value),
        ][..], stdout()).expect("can always print");
        sleep(Duration::new(1, 0));
    }
}
