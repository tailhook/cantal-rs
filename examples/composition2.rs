extern crate libcantal;
extern crate env_logger;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;

use std::env;
use std::io::stdout;
use std::time::Duration;
use std::thread::sleep;

use libcantal::{Collection, Counter, Integer, Value, start, print};
use serde_json::Value as Json;


lazy_static! {
    static ref COUNTER: Counter = Counter::new();
    static ref INTEGER: Integer = Integer::new();
}

fn counters() -> Vec<(Json, &'static Value)> {
    vec![
        (json!({"metric": "counter"}), &*COUNTER),
    ]
}

fn integers() -> Vec<(Json, &'static Value)> {
    vec![
        (json!({"metric": "integer"}), &*INTEGER),
    ]
}

fn metrics() -> Box<Collection> {
    Box::new(vec![counters(), integers()])
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init().unwrap();

    let met = metrics();
    let _coll = start(&met).expect("cantal works");
    loop {
        COUNTER.incr(1);
        INTEGER.set((COUNTER.get() / 7) as i64);
        print(&met, stdout()).expect("can always print");
        sleep(Duration::new(1, 0));
    }
}
