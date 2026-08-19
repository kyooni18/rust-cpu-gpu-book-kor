use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

// 128바이트 경계로 정렬한 컨테이너입니다. 두 카운터가 항상 같은
// 캐시 라인에 놓입니다(라인 폭이 64바이트든 128바이트든 동일합니다)
#[repr(align(128))]
struct SameLine([AtomicU64; 2]);

// 하나가 128바이트를 차지하는 컨테이너입니다. 두 개를 나란히 두면
// 카운터가 항상 서로 다른 캐시 라인에 놓입니다
#[repr(align(128))]
struct Padded(AtomicU64);

fn main() {
    println!("사용 가능한 병렬성: {:?}", thread::available_parallelism());

    let iters = 50_000_000u64;

    // (1) 같은 캐시 라인에 있는 두 카운터
    let same = SameLine([AtomicU64::new(0), AtomicU64::new(0)]);
    let start = Instant::now();
    thread::scope(|s| {
        for c in &same.0 {
            s.spawn(move || {
                for _ in 0..iters {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    println!("같은 라인: {:>9.3?}", start.elapsed());

    // (2) 서로 다른 캐시 라인에 있는 두 카운터
    let padded = [Padded(AtomicU64::new(0)), Padded(AtomicU64::new(0))];
    let start = Instant::now();
    thread::scope(|s| {
        for p in &padded {
            s.spawn(move || {
                for _ in 0..iters {
                    p.0.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    println!("다른 라인: {:>9.3?}", start.elapsed());
}
