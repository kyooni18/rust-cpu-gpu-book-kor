fn main() {
    println!("0.1 + 0.2 == 0.3 : {}", 0.1 + 0.2 == 0.3);
    println!("0.1 + 0.2        = {:.20}", 0.1 + 0.2);
    println!("0.3              = {:.20}", 0.3);
    println!();
    // 0.1로 저장된 64비트의 실제 내용
    println!("0.1의 비트열:");
    let bits = 0.1f64.to_bits();
    println!("  부호: {:b}", bits >> 63);
    println!("  지수: {:011b}", (bits >> 52) & 0x7FF);
    println!("  가수: {:052b}", bits & ((1 << 52) - 1));
}
