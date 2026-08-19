fn main() {
    let v = vec![10u8, 20, 30];
    let i = std::hint::black_box(7usize); // 범위를 벗어난 인덱스
    // 검사가 있다면 panic하지만……
    let x = unsafe { *v.get_unchecked(i) };
    println!("v[{i}] = {x} (?!)");
}
