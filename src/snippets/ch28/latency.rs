use std::collections::HashMap;
use std::time::Instant;

fn main() {
    // HashMap에 100만 번 insert하면서 각 작업을 개별 측정합니다
    let n = 1_000_000u64;
    let mut map: HashMap<u64, u64> = HashMap::new();
    let mut lat_ns: Vec<u64> = Vec::with_capacity(n as usize);

    for i in 0..n {
        let start = Instant::now();
        map.insert(i, i);
        lat_ns.push(start.elapsed().as_nanos() as u64);
    }

    lat_ns.sort_unstable();
    let pick = |p: f64| lat_ns[((n as f64 - 1.0) * p) as usize];
    let mean = lat_ns.iter().sum::<u64>() as f64 / n as f64;
    println!("평균   : {mean:8.0} ns");
    println!("중앙값 : {:8} ns", pick(0.50));
    println!("p99    : {:8} ns", pick(0.99));
    println!("p99.9  : {:8} ns", pick(0.999));
    println!("최대   : {:8} ns  ← 이상값(주된 원인은 리해시=전체 복사)", lat_ns[n as usize - 1]);
}
