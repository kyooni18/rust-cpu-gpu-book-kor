use std::hint::black_box;
use std::time::Instant;

use std::ffi::c_long;

// C 라이브러리 함수를 직접 선언합니다(labs = C의 long 절댓값 함수).
// C의 long 폭은 OS에 따라 다릅니다(64비트 Linux/macOS에서는 64비트, Windows에서는 32비트).
// 따라서 대응하는 Rust 타입인 c_long을 사용합니다
unsafe extern "C" {
    fn labs(x: c_long) -> c_long;
}

fn main() {
    let n = 100_000_000i64;

    // (1) Rust의 .abs(): 인라이닝되고 벡터화될 수도 있습니다
    let start = Instant::now();
    let mut sum = 0i64;
    for i in -n / 2..n / 2 {
        sum = sum.wrapping_add(black_box(i).abs());
    }
    println!("Rust abs : {:>9.3?} (sum={sum})", start.elapsed());

    // (2) C 함수 FFI 호출: 매번 호출 경계를 넘습니다
    let start = Instant::now();
    let mut sum = 0i64;
    for i in -n / 2..n / 2 {
        sum = sum.wrapping_add(unsafe { labs(black_box(i) as c_long) } as i64);
    }
    println!("C labs   : {:>9.3?} (sum={sum})", start.elapsed());
}
