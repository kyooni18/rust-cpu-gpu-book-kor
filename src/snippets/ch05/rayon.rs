use rayon::prelude::*;
use std::time::Instant;

// 콜라츠 수열: n이 1이 될 때까지의 단계 수를 셉니다(원소 하나당 작업이 무거운 예)
fn collatz_steps(mut n: u64) -> u64 {
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

fn main() {
    let range = 1u64..2_000_000;

    // Rayon 스레드 풀은 처음 사용할 때 만들어지므로
    // 측정 전에 한 번 실행해 준비합니다(워밍업)
    rayon::join(|| (), || ());

    let start = Instant::now();
    let total: u64 = range.clone().map(collatz_steps).sum();
    println!("순차: {:>9.3?} (total={total})", start.elapsed());

    // 바뀐 부분은 into_par_iter()뿐입니다
    let start = Instant::now();
    let total: u64 = range.into_par_iter().map(collatz_steps).sum();
    println!("병렬: {:>9.3?} (total={total})", start.elapsed());
}
