use std::time::Instant;

fn main() {
    // u64 800만 개 = 64MB. 캐시에 들어가지 않을 만큼 크게 만듭니다
    let n = 8_000_000;
    let data: Vec<u64> = (0..n as u64).collect();

    // (1) 모든 원소를 순서대로 읽습니다
    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..n {
        sum = sum.wrapping_add(data[i]);
    }
    println!("모든 원소 (읽기 {n}회): {:>9.3?}", start.elapsed());
    assert!(sum != 0);

    // (2) 원소 8개마다 하나씩 읽습니다(읽기 횟수는 1/8)
    let start = Instant::now();
    let mut sum = 0u64;
    let mut i = 0;
    while i < n {
        sum = sum.wrapping_add(data[i]);
        i += 8;
    }
    println!("8개 간격 (읽기 {}회): {:>9.3?}", n / 8, start.elapsed());
    assert!(sum != 0);
}
