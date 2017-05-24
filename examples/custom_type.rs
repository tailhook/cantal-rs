extern crate libcantal;
extern crate env_logger;

#[macro_use] extern crate lazy_static;

use std::env;
use std::io::stdout;
use std::time::Duration;
use std::thread::sleep;

use libcantal::{Collection, Counter, Integer, Value, start, print};
use libcantal::{Name, NameVisitor};


lazy_static! {
    static ref COUNTER: Counter = Counter::new();
    static ref INTEGER: Integer = Integer::new();
}

// Zero-allocating types for metrics
struct Metric(&'static str);

impl Name for Metric {
    fn get(&self, key: &str) -> Option<&str> {
        if key == "metric" {
            return Some(self.0);
        } else {
            return None;
        }
    }
    fn visit(&self, s: &mut NameVisitor) {
        s.visit_pair("metric", self.0);
    }
}

fn counters() -> Vec<(Metric, &'static Value)> {
    vec![
        (Metric("counter"), &*COUNTER),
    ]
}

fn integers() -> Vec<(Metric, &'static Value)> {
    vec![
        (Metric("integer"), &*INTEGER),
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
