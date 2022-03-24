use prof_rs::cpu::{active, deactive, dump};
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    let _ = active(1000);
    let mut map = HashMap::new();
    for i in 0..1000 {
        map.insert(i, i);
        println!("insert key = {}, val = {}", i, i);
        std::thread::sleep(Duration::from_millis(1));
    }
    for i in 0..1000 {
        let val = map.get(&i);
        println!("when key = {}, get val = {:?}", i, val);
    }
    let _ = dump("cpu");
    let _ = deactive();
}
