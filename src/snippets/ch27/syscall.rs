use std::hint::black_box;
use std::time::Instant;

unsafe extern "C" {
    fn getpid() -> i32;
}

#[inline(never)]
fn plain_function(x: i32) -> i32 {
    black_box(x + 1)
}

fn main() {
    let n = 5_000_000;

    let start = Instant::now();
    let mut acc = 0i64;
    for i in 0..n {
        acc += plain_function(i) as i64;
    }
    let t = start.elapsed();
    println!(
        "일반 함수 호출      : {t:>9.3?} ({:5.1}ns/회, acc={acc})",
        t.as_nanos() as f64 / n as f64
    );

    let start = Instant::now();
    let mut acc = 0i64;
    for _ in 0..n {
        acc += unsafe { getpid() } as i64;
    }
    let t = start.elapsed();
    println!(
        "getpid 시스템 콜   : {t:>7.3?} ({:5.1}ns/회, acc={acc})",
        t.as_nanos() as f64 / n as f64
    );
}
