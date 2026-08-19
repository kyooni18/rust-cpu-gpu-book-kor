use std::time::Instant;

// 인라이닝을 금지한 작은 함수
#[inline(never)]
fn add_never(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

// 일반적인 작은 함수(인라이닝 여부는 컴파일러가 판단합니다)
fn add_auto(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

fn main() {
    let n = 100_000_000u64;

    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..n {
        sum = add_never(sum, i);
    }
    println!("inline(never): {:>9.3?} (sum={sum})", start.elapsed());

    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..n {
        sum = add_auto(sum, i);
    }
    println!("자동 판단    : {:>9.3?} (sum={sum})", start.elapsed());
}
