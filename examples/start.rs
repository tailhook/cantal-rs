extern crate libcantal;
extern crate env_logger;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;

use std::env;
use std::time::Duration;
use std::thread::sleep;

use libcantal::{Counter, Integer, Value, start};


lazy_static! {
    static ref COUNTER: Counter = Counter::new();
    static ref INTEGER: Integer = Integer::new();
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init().unwrap();

    let metrics = [
        (json!({"metric": "counter"}), &*COUNTER as &Value),
        (json!({"metric": "integer"}), &*INTEGER as &Value),
    ];
    let _coll = start(&metrics[..]).expect("cantal works");
    loop {
        COUNTER.incr(1);
        INTEGER.set((COUNTER.get() / 7) as i64);
        sleep(Duration::new(1, 0));
    }
}
