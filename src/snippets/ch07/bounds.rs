use std::time::Instant;

fn main() {
    let n = 10_000_000;
    let v: Vec<i32> = (0..n as i32).collect();
    let passes = 20;

    // (1) 인덱스 접근. v[i]가 범위를 벗어나면 panic합니다
    let start = Instant::now();
    let mut total = 0i64;
    for _ in 0..passes {
        let mut s = 0i32;
        for i in 0..n {
            s = s.wrapping_add(v[i]);
        }
        total += s as i64;
    }
    println!("인덱스 v[i]      : {:>9.3?} (total={total})", start.elapsed());

    // (2) 이터레이터
    let start = Instant::now();
    let mut total = 0i64;
    for _ in 0..passes {
        let mut s = 0i32;
        for &x in v.iter() {
            s = s.wrapping_add(x);
        }
        total += s as i64;
    }
    println!("이터레이터       : {:>9.3?} (total={total})", start.elapsed());

    // (3) 실행 시 결정되는 길이 m까지 인덱스로 접근합니다
    // black_box로 "컴파일 시점에는 값을 알 수 없는" 상황을 만듭니다
    let m = std::hint::black_box(n - 1);
    assert!(m <= v.len());
    let start = Instant::now();
    let mut total = 0i64;
    for _ in 0..passes {
        let mut s = 0i32;
        for i in 0..m {
            s = s.wrapping_add(v[i]);
        }
        total += s as i64;
    }
    println!("인덱스 0..m      : {:>9.3?} (total={total})", start.elapsed());

    // (4) 먼저 슬라이스를 만든 뒤 이터레이터를 사용합니다
    let start = Instant::now();
    let mut total = 0i64;
    for _ in 0..passes {
        let mut s = 0i32;
        for &x in &v[..m] {
            s = s.wrapping_add(x);
        }
        total += s as i64;
    }
    println!("슬라이스 &v[..m]: {:>9.3?} (total={total})", start.elapsed());
}
