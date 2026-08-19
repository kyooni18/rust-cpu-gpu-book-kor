use std::time::Instant;

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

// 내부 동작이 거의 같은 함수 16개를 준비합니다
macro_rules! ops {
    ($($name:ident, $k:expr;)*) => {
        $(#[inline(never)] fn $name(x: u64) -> u64 { x.wrapping_mul(2).wrapping_add($k) })*
        const OPS: [fn(u64) -> u64; 16] = [$($name),*];
    };
}
ops!(f0,0; f1,1; f2,2; f3,3; f4,4; f5,5; f6,6; f7,7; f8,8; f9,9; f10,10; f11,11; f12,12; f13,13; f14,14; f15,15;);

fn main() {
    let n = 20_000_000;
    let mut state = 0x2545_F491_4F6C_DD1D_u64;

    // 호출 대상 순서: 규칙적(순환) vs 무작위
    let regular: Vec<u8> = (0..n).map(|i| (i % 16) as u8).collect();
    let random: Vec<u8> = (0..n).map(|_| (xorshift(&mut state) % 16) as u8).collect();

    for (name, idx) in [("규칙적", &regular), ("무작위", &random)] {
        let start = Instant::now();
        let mut x = 0u64;
        for &i in idx.iter() {
            x = OPS[i as usize](x);
        }
        println!("{name}: {:>9.3?} (x={x})", start.elapsed());
    }
}
