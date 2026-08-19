use std::mem::size_of_val;

async fn tiny() -> u64 {
    1 + 1
}

// 4KB 버퍼를 .await 너머까지 유지합니다
async fn holds_buffer() -> u64 {
    let buf = [7u8; 4096];
    tokio::task::yield_now().await; // 여기에서 중단될 수 있습니다
    buf.iter().map(|&b| b as u64).sum()
}

// 같은 버퍼라도 .await 전에 사용을 끝냅니다
async fn drops_before_await() -> u64 {
    let sum = {
        let buf = [7u8; 4096];
        buf.iter().map(|&b| b as u64).sum()
    };
    tokio::task::yield_now().await;
    sum
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let f1 = tiny();
    let f2 = holds_buffer();
    let f3 = drops_before_await();
    println!("tiny Future              : {:>5} bytes", size_of_val(&f1));
    println!("버퍼가 await를 넘는 Future: {:>5} bytes", size_of_val(&f2));
    println!("await 전에 놓는 Future    : {:>5} bytes", size_of_val(&f3));
    println!("결과: {} {} {}", f1.await, f2.await, f3.await);
}
