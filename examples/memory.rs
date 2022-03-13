use prof_rs::jemalloc_mem::{active, deactive, dump};

/// first step: export _RJEM_MALLOC_CONF=prof:true,prof_active:false
/// second step: cargo run --example memory

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let _ = active();
    memory_leak();
    let _ = dump("memory");
    let _ = deactive();
}

fn memory_leak() {
    for _ in 0..10 {
        Box::leak(Box::new(Vec::<u8>::with_capacity(100)));
    }
}