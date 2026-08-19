use std::hint::black_box;

fn main() {
    // black_box로 "실행 시점에만 알 수 있는 값"을 만듭니다
    // 상수 그대로 두면 컴파일 시점에 오버플로를 발견해 빌드가 중단됩니다
    let a: i32 = black_box(i32::MAX);
    let b = a + 1;
    println!("i32::MAX + 1 = {b}");
}
