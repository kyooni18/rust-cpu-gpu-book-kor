use std::time::Instant;

fn main() {
    let n: u64 = 100_000_000; // 1억 번
    let start = Instant::now();

    let mut sum: u64 = 0;
    for i in 0..n {
        sum = sum.wrapping_add(i);
    }

    let elapsed = start.elapsed();
    println!("합계: {sum}");
    println!("경과 시간: {elapsed:?}");
    println!(
        "1초당 약 {:.1}억 번의 덧셈",
        n as f64 / elapsed.as_secs_f64() / 1e8
    );
}
