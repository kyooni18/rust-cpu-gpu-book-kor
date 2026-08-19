use std::time::{Duration, Instant};

// 워커가 하나인 런타임에서
// 무거운 동기 작업이 함께 실행되는 다른 태스크에 어떤 영향을 주는지 확인합니다
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    // (1) 태스크 A가 std::thread::sleep으로 워커 스레드 자체를 멈춥니다
    rt.block_on(async {
        let start = Instant::now();
        let b = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            start.elapsed()
        });
        // 태스크 B가 먼저 타이머를 등록할 수 있도록 한 번 실행을 양보합니다
        tokio::task::yield_now().await;
        let a = tokio::spawn(async {
            std::thread::sleep(Duration::from_millis(300)); // 블로킹!
        });
        a.await.unwrap();
        let b_done = b.await.unwrap();
        println!("(1) 블로킹 작업 동시 실행: 50ms 태스크 B 완료 = {b_done:?}");
    });

    // (2) 무거운 작업을 spawn_blocking으로 전용 스레드에 보냅니다
    rt.block_on(async {
        let start = Instant::now();
        let b = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            start.elapsed()
        });
        tokio::task::yield_now().await;
        let a = tokio::task::spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(300));
        });
        a.await.unwrap();
        let b_done = b.await.unwrap();
        println!("(2) spawn_blocking 사용      : 50ms 태스크 B 완료 = {b_done:?}");
    });
}
