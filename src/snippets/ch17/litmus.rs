use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering::*};
use std::thread;

// 스토어→로드 재정렬을 관찰하는 리트머스 테스트입니다.
// 두 스레드가 동시에 "자기 변수에 1을 쓴 뒤 상대 변수 읽기"를 수행합니다.
// 명령어 순서가 그대로 관찰된다면 두 스레드가 모두 0을 읽는 결과는 나올 수 없습니다.
fn litmus(trials: usize, seqcst: bool) -> usize {
    let x = AtomicI32::new(0);
    let y = AtomicI32::new(0);
    let r2 = AtomicI32::new(0);
    let bar = AtomicUsize::new(0);

    // 두 스레드용 배리어: 양쪽이 모두 도착할 때까지 기다립니다
    let barrier = |target: usize| {
        bar.fetch_add(1, AcqRel);
        while bar.load(Acquire) < target {
            std::hint::spin_loop();
        }
    };

    let mut both_zero = 0;
    thread::scope(|s| {
        // 스레드 B
        s.spawn(|| {
            for t in 0..trials {
                y.store(0, Relaxed);
                barrier(4 * t + 2); // 준비 완료를 맞춥니다
                if seqcst {
                    y.store(1, SeqCst);
                    r2.store(x.load(SeqCst), Relaxed);
                } else {
                    y.store(1, Relaxed);
                    r2.store(x.load(Relaxed), Relaxed);
                }
                barrier(4 * t + 4); // 실행 완료를 맞춥니다
            }
        });
        // 스레드 A(이 스레드가 결과 판정도 수행합니다)
        for t in 0..trials {
            x.store(0, Relaxed);
            barrier(4 * t + 2);
            let r1 = if seqcst {
                x.store(1, SeqCst);
                y.load(SeqCst)
            } else {
                x.store(1, Relaxed);
                y.load(Relaxed)
            };
            barrier(4 * t + 4);
            if r1 == 0 && r2.load(Relaxed) == 0 {
                both_zero += 1;
            }
        }
    });
    both_zero
}

fn main() {
    let trials = 200_000;
    println!(
        "Relaxed: {trials}번 중 {:>6}번 모두 0 (재정렬 관찰)",
        litmus(trials, false)
    );
    println!(
        "SeqCst : {trials}번 중 {:>6}번 모두 0",
        litmus(trials, true)
    );
}
