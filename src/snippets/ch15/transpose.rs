use std::time::Instant;

fn main() {
    let n = 2048;
    let src: Vec<f32> = (0..n * n).map(|i| i as f32).collect();
    let mut dst = vec![0.0f32; n * n];

    // (1) 단순 전치: dst 쓰기가 열 방향(스트라이드 n)으로 진행됩니다
    let start = Instant::now();
    for i in 0..n {
        for j in 0..n {
            dst[j * n + i] = src[i * n + j];
        }
    }
    println!("단순 전치          : {:>9.3?} (check={})", start.elapsed(), dst[123 * n + 45]);

    // (2) 32×32 블록 단위로 전치: 읽기와 쓰기가 캐시 안에서 더 잘 재사용됩니다
    let mut dst2 = vec![0.0f32; n * n];
    let b = 32;
    let start = Instant::now();
    for bi in (0..n).step_by(b) {
        for bj in (0..n).step_by(b) {
            for i in bi..bi + b {
                for j in bj..bj + b {
                    dst2[j * n + i] = src[i * n + j];
                }
            }
        }
    }
    println!("블록 전치(32×32)   : {:>9.3?} (check={})", start.elapsed(), dst2[123 * n + 45]);
    assert!(dst == dst2);
}
