use std::sync::mpsc;
use std::time::Instant;

fn main() {
    // 두 스레드가 채널로 값을 주고받습니다. 한 번 왕복할 때 스레드 깨우기가 두 번 발생합니다
    let rounds = 100_000;
    let (tx1, rx1) = mpsc::channel::<u64>();
    let (tx2, rx2) = mpsc::channel::<u64>();

    let handle = std::thread::spawn(move || {
        for _ in 0..rounds {
            let v = rx1.recv().unwrap();
            tx2.send(v + 1).unwrap();
        }
    });

    let start = Instant::now();
    let mut v = 0u64;
    for _ in 0..rounds {
        tx1.send(v).unwrap();
        v = rx2.recv().unwrap();
    }
    let t = start.elapsed();
    handle.join().unwrap();
    println!(
        "{rounds}회 왕복: {t:?} (왕복 1회당 {:5.2}µs, v={v})",
        t.as_nanos() as f64 / rounds as f64 / 1000.0
    );
}
