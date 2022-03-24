use std::time::Duration;

use prof_rs::jemalloc_mem::{active, deactive, dump};

/// This is a simple example about how to generate memory flamegraph in rust (only test on linux)
/// 1 export _RJEM_MALLOC_CONF=prof:true,prof_active:false
/// 2 cargo run --example memory
/// 3 ./tool/jeprof ./target/debug/examples/memory memory.2022-03-24_14:14:33.out --collapsed | ./tool/flamegraph.pl > flamegraph.svg
/// Note: jeprof was build from jemalloc dev branch (https://github.com/jemalloc/jemalloc/pull/1984) 
/// Note: flamegraph.pl is from (https://github.com/brendangregg/FlameGraph.git)

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let _ = active();
    memory_leak();
    let _ = dump("memory");
    let _ = deactive();
}

fn memory_leak() {
    for _ in 0..10000 {
        Box::leak(Box::new(Vec::<u8>::with_capacity(100)));
    }
}
