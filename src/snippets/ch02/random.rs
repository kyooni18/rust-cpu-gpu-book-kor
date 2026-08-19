use std::time::Instant;

// 의사 난수(외부 크레이트 없이 사용하기 위한 간단한 구현)
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn main() {
    let n: usize = 16_000_000; // u32 기준 64MB. 캐시에 들어가지 않을 만큼 큰 크기

    // next[i] = "다음에 따라갈 인덱스". 먼저 순차적인 형태를 만듭니다
    let seq: Vec<u32> = (0..n as u32).map(|i| (i + 1) % n as u32).collect();

    // 모든 원소를 무작위 순서로 한 번씩 순회하는 고리를 만듭니다(Sattolo 알고리즘).
    // i -> rand[i]를 따라가면 모든 원소를 한 번씩 지나 다시 돌아옵니다
    let mut rand: Vec<u32> = (0..n as u32).collect();
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    for i in (1..n).rev() {
        let j = (xorshift(&mut state) % i as u64) as usize;
        rand.swap(i, j);
    }

    // 두 경우 모두 배열을 정확히 n번 따라간다는 점은 같습니다
    for (name, next) in [("순차 접근", &seq), ("무작위 접근", &rand)] {
        let start = Instant::now();
        let mut pos = 0u32;
        for _ in 0..n {
            pos = next[pos as usize];
        }
        println!("{name}: {:>9.3?} (마지막 위치 {pos})", start.elapsed());
    }
}
