extern crate libcantal;
#[macro_use] extern crate lazy_static;

use std::time::Duration;
use std::thread::sleep;

use libcantal::Counter;


lazy_static! {
    static ref COUNTER: Counter = Counter::new();
}

fn main() {
    loop {
        COUNTER.incr(1);
        println!("Counter value: {}", COUNTER.get());
        sleep(Duration::new(1, 0));
    }
}
