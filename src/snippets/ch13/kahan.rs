fn main() {
    // 0.1을 1,000만 번 더합니다. 정확한 값은 1,000,000입니다
    let n = 10_000_000;

    // (1) 단순하게 더합니다
    let mut plain = 0.0f32;
    for _ in 0..n {
        plain += 0.1;
    }

    // (2) Kahan 합계: 잃어버린 오차 c를 기억했다가 다음 덧셈에서 보정합니다
    let mut sum = 0.0f32;
    let mut c = 0.0f32;
    for _ in 0..n {
        let y = 0.1 - c;
        let t = sum + y;
        c = (t - sum) - y; // 이 한 줄이 반올림으로 잃은 부분을 회수합니다
        sum = t;
    }

    // (3) 누산만 f64로 수행합니다
    let mut wide = 0.0f64;
    for _ in 0..n {
        wide += 0.1f32 as f64;
    }

    println!("단순 f32   : {plain}");
    println!("Kahan f32  : {sum}");
    println!("f64 누산   : {wide:.3}");
    println!("정확한 값  : 1000000");
}
