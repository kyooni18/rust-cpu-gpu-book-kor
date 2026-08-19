use std::hint::black_box;
use std::time::Instant;

fn main() {
    let size = 128 * 1024 * 1024;

    let start = Instant::now();
    let mut zeroed = vec![0u8; size];
    println!("vec![0u8; 128MB] 할당       : {:>11.3?}", start.elapsed());

    let start = Instant::now();
    let ones = vec![1u8; size];
    black_box(&ones);
    println!("vec![1u8; 128MB] 할당       : {:>11.3?}", start.elapsed());
    drop(ones);

    let start = Instant::now();
    for i in (0..size).step_by(4096) {
        zeroed[i] = 1;
    }
    black_box(&zeroed);
    println!("첫 번째 쓰기(4KiB 간격)     : {:>10.3?}", start.elapsed());

    let start = Instant::now();
    for i in (0..size).step_by(4096) {
        zeroed[i] = 2;
    }
    black_box(&zeroed);
    println!("두 번째 쓰기(같은 위치)     : {:>10.3?}", start.elapsed());
}
