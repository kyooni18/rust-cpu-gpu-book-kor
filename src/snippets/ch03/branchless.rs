use std::time::Instant;

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

    // (1) if를 산술 연산으로 바꿉니다: 조건을 0/1 값으로 변환해 곱합니다
    for (name, data) in [("정렬 안 됨", &unsorted), ("정렬됨", &sorted)] {
        let start = Instant::now();
        let mut sum = 0u64;
        for &v in data.iter() {
            sum += u64::from(v >= 128) * v as u64;
        }
        println!("산술   {name}: {:>9.3?} (sum={sum})", start.elapsed());
    }

    // (2) 이터레이터로 작성합니다
    for (name, data) in [("정렬 안 됨", &unsorted), ("정렬됨", &sorted)] {
        let start = Instant::now();
        let sum: u64 = data
            .iter()
            .filter(|&&v| v >= 128)
            .map(|&v| v as u64)
            .sum();
        println!("filter {name}: {:>9.3?} (sum={sum})", start.elapsed());
    }
}
