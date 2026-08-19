use std::time::Instant;

fn slow_square(x: u64) -> u64 {
    // 의도적으로 무겁게 만든 계산
    let mut acc = x;
    for _ in 0..100 {
        acc = acc.wrapping_mul(acc) ^ x;
    }
    acc
}

fn main() {
    let v: Vec<u64> = (0..1_000_000).collect();

    // (1) map만 만들고 어디에서도 소비하지 않습니다
    let start = Instant::now();
    let _it = v.iter().map(|&x| slow_square(x));
    println!("map만 생성: {:>12.3?}", start.elapsed());

    // (2) sum으로 끝까지 실행합니다
    let start = Instant::now();
    let total: u64 = v.iter().map(|&x| slow_square(x)).sum();
    println!("sum까지 실행: {:>12.3?} (total={total})", start.elapsed());
}
