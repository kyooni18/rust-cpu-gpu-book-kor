//! 25장: 직접 작성한 행렬 곱과 벤더 라이브러리(Accelerate/BLAS)를 비교합니다.
//! 실행: cd examples && cargo run --release -p ch25-gemm-lib [-- n]
//! macOS 전용입니다. 다른 OS에서는 OpenBLAS 같은 라이브러리로 바꿔 생각하세요.

use rayon::prelude::*;
use std::time::Instant;

// BLAS 행렬 곱 C = alpha*A*B + beta*C (FFI 선언, 20장)
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn cblas_sgemm(
        order: i32,      // 101 = RowMajor
        trans_a: i32,    // 111 = NoTrans
        trans_b: i32,
        m: i32,
        n: i32,
        k: i32,
        alpha: f32,
        a: *const f32,
        lda: i32,
        b: *const f32,
        ldb: i32,
        beta: f32,
        c: *mut f32,
        ldc: i32,
    );
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn gflops(n: usize, secs: f64) -> f64 {
    (2.0 * (n as f64).powi(3)) / secs / 1e9
}

/// 12장의 CPU 최선 버전(ikj + rayon)
fn matmul_par(a: &[f32], b: &[f32], c: &mut [f32], n: usize) {
    c.par_chunks_mut(n).enumerate().for_each(|(i, c_row)| {
        c_row.fill(0.0);
        for k in 0..n {
            let aik = a[i * n + k];
            let b_row = &b[k * n..k * n + n];
            for j in 0..n {
                c_row[j] += aik * b_row[j];
            }
        }
    });
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024);
    assert!(n > 0 && n <= 8192, "n은 1..=8192 범위여야 합니다");
    let _ = n.checked_mul(n).expect("n*n 계산이 오버플로했습니다");
    let _ = i32::try_from(n).expect("n을 i32로 표현할 수 없습니다");
    println!("n = {n}\n");

    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let a: Vec<f32> = (0..n * n)
        .map(|_| (xorshift(&mut state) % 1000) as f32 / 1000.0)
        .collect();
    let b: Vec<f32> = (0..n * n)
        .map(|_| (xorshift(&mut state) % 1000) as f32 / 1000.0)
        .collect();
    let mut c = vec![0.0f32; n * n];

    // 직접 작성한 최선의 CPU 버전(12장)
    let start = Instant::now();
    matmul_par(&a, &b, &mut c, n);
    let t = start.elapsed();
    println!("직접 구현 ikj+rayon: {t:>9.3?} ({:7.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    let reference = c.clone();

    // Accelerate(BLAS의 sgemm)
    #[cfg(target_os = "macos")]
    {
        let mut c2 = vec![0.0f32; n * n];
        // 워밍업 + 측정
        for round in 0..2 {
            c2.fill(0.0);
            let start = Instant::now();
            unsafe {
                cblas_sgemm(
                    101, 111, 111,
                    n as i32, n as i32, n as i32,
                    1.0,
                    a.as_ptr(), n as i32,
                    b.as_ptr(), n as i32,
                    0.0,
                    c2.as_mut_ptr(), n as i32,
                );
            }
            if round == 1 {
                let t = start.elapsed();
                println!("Accelerate sgemm  : {t:>9.3?} ({:7.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
            }
        }
        // 반올림 차이를 허용하면서 결과를 검증합니다
        let max_diff = reference
            .iter()
            .zip(&c2)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        println!("최대 오차: {max_diff:e}");
    }
}
