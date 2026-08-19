use std::time::Instant;

fn main() {
    // 64개 위치를 반복해서 순회하며 읽습니다. 실제로 접근하는 데이터는 64 × 8바이트뿐입니다
    let slots = 64;
    let rounds = 2_000_000;

    for (name, stride) in [("4096바이트 간격(2의 거듭제곱)", 4096usize), ("4160바이트 간격(+64 이동)", 4160)] {
        let buf = vec![1u8; slots * stride + 8];
        let start = Instant::now();
        let mut sum = 0u64;
        for _ in 0..rounds {
            for i in 0..slots {
                let p = i * stride;
                let v = u64::from_ne_bytes(buf[p..p + 8].try_into().unwrap());
                sum = sum.wrapping_add(v);
            }
        }
        println!("{name}: {:>9.3?} (sum={sum})", start.elapsed());
    }
}
