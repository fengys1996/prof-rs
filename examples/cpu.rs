use std::collections::HashMap;
use std::time::Duration;
use prof_rs::cpu::{active, deactive, dump};

fn main() {
    let _ = active(100);
    let mut map = HashMap::new();
    for i in 0..10000 {
        map.insert(i ,i );
        std::thread::sleep(Duration::from_millis(1));
    }
    for i in 0..10000 {
        let val = map.get(&i);
        println!("{:?}", val);
    }
    dump("cpu");
    let _ = deactive();
}