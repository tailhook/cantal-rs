extern crate libcantal;
#[macro_use] extern crate lazy_static;

use std::time::Duration;
use std::thread::sleep;

use libcantal::Integer;


lazy_static! {
    static ref INTEGER: Integer = Integer::new();
}

fn main() {
    INTEGER.set(23);
    loop {
        INTEGER.incr(3);
        println!("Integer value: {}", INTEGER.get());
        sleep(Duration::new(1, 0));
        INTEGER.decr(1);
        println!("Integer value: {}", INTEGER.get());
        sleep(Duration::new(1, 0));
    }
}
