use std::hint::black_box;
use std::time::Instant;

fn main() {
    let n = 10_000_000usize;
    let src: Vec<u32> = (0..n as u32).collect();

    // (1) 빈 Vec에 push: 용량이 부족해질 때마다 재할당과 복사가 발생합니다
    let start = Instant::now();
    let mut out = Vec::new();
    for &v in src.iter() {
        out.push(v as u64 * 2);
    }
    black_box(&out);
    println!("Vec::new + push     : {:>9.3?}", start.elapsed());
    drop(out);

    // (2) 필요한 용량을 먼저 확보한 뒤 push합니다
    let start = Instant::now();
    let mut out = Vec::with_capacity(n);
    for &v in src.iter() {
        out.push(v as u64 * 2);
    }
    black_box(&out);
    println!("with_capacity + push: {:>9.3?}", start.elapsed());
    drop(out);

    // (3) 이터레이터에서 collect합니다
    let start = Instant::now();
    let out: Vec<u64> = src.iter().map(|&v| v as u64 * 2).collect();
    black_box(&out);
    println!("collect             : {:>9.3?}", start.elapsed());
}
