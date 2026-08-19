use std::time::Instant;

// 의사 난수(외부 크레이트 없이 사용)
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn main() {
    let n = 10_000_000;
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let unsorted: Vec<u8> = (0..n)
        .map(|_| (xorshift(&mut state) & 0xFF) as u8)
        .collect();
    let mut sorted = unsorted.clone();
    sorted.sort_unstable();

    // 내용은 같은 데이터에 완전히 같은 코드를 적용하고 순서만 바꿔 비교합니다
    for (name, data) in [("정렬 안 됨", &unsorted), ("정렬됨", &sorted)] {
        let start = Instant::now();
        let mut sum = 0u64;
        for &v in data.iter() {
            if v >= 128 {
                sum += v as u64;
            }
        }
        println!("{name}: {:>9.3?} (sum={sum})", start.elapsed());
    }
}
