use std::mem::{align_of, size_of};

// 필드 배치 순서는 컴파일러에 맡깁니다(기본값)
#[allow(dead_code)]
struct Auto {
    a: u8,
    b: u64,
    c: u16,
}

// C와 같은 규칙: 선언 순서를 유지하면서 정렬 조건에 맞춰 배치합니다
#[allow(dead_code)]
#[repr(C)]
struct CLayout {
    a: u8,
    b: u64,
    c: u16,
}

fn main() {
    println!(
        "기본값   : size = {:2} bytes, align = {} bytes",
        size_of::<Auto>(),
        align_of::<Auto>()
    );
    println!(
        "#[repr(C)]: size = {:2} bytes, align = {} bytes",
        size_of::<CLayout>(),
        align_of::<CLayout>()
    );
}
