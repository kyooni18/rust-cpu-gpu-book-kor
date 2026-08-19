use std::time::Instant;

fn main() {
    // 100만 원소 = 8MB. 캐시에 대부분 들어가도록 해 메모리 대기 영향을 줄입니다
    let n = 1_000_000;
    let data: Vec<f64> = (0..n).map(|i| (i % 100) as f64 * 0.01).collect();
    let passes = 20;

    // 누산기 1개: 이전 덧셈이 끝나야 다음 덧셈을 시작할 수 있습니다
    let start = Instant::now();
    let mut sum = 0.0f64;
    for _ in 0..passes {
        for &v in data.iter() {
            sum += v;
        }
    }
    println!("누산기 1개: {:>9.3?} (sum={sum:.0})", start.elapsed());

    // 누산기 4개: 서로 의존하지 않는 네 개의 덧셈 체인이 병렬로 진행될 수 있습니다
    let start = Instant::now();
    let mut sum = 0.0f64;
    for _ in 0..passes {
        let mut s = [0.0f64; 4];
        let mut chunks = data.chunks_exact(4);
        for c in &mut chunks {
            s[0] += c[0];
            s[1] += c[1];
            s[2] += c[2];
            s[3] += c[3];
        }
        sum += s.iter().sum::<f64>() + chunks.remainder().iter().sum::<f64>();
    }
    println!("누산기 4개: {:>9.3?} (sum={sum:.0})", start.elapsed());
}
