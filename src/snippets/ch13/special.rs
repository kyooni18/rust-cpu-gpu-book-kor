fn main() {
    // NaN은 ==, <, >, <=, >= 비교에 모두 false를 반환합니다(!=만 true입니다)
    let nan = f64::NAN;
    println!("NaN == NaN : {}", nan == nan);
    println!("NaN <  1.0 : {}", nan < 1.0);
    println!("NaN >  1.0 : {}", nan > 1.0);

    // 그래서 f64는 Ord를 구현하지 않아 sort()를 직접 사용할 수 없습니다.
    // 전체 순서가 필요하면 total_cmp를 사용합니다
    let mut v = vec![3.0, f64::NAN, 1.0, 2.0];
    v.sort_by(f64::total_cmp);
    println!("total_cmp로 정렬: {v:?}");

    println!();
    // 정규화 수의 최솟값보다 작아져도 정밀도를 낮추며 더 작은 값을 표현할 수 있습니다
    println!("f32 최소 정규화 수        : {:e}", f32::MIN_POSITIVE);
    println!("8로 나눈 값(서브노멀 수) : {:e}", f32::MIN_POSITIVE / 8.0);
    println!("표현 가능한 최소 양수     : {:e}", f32::from_bits(1));

    println!();
    // 부호 있는 0: 비교하면 같지만 나눗셈에서는 부호가 드러납니다
    println!("0.0 == -0.0 : {}", 0.0f64 == -0.0f64);
    println!("1.0 /  0.0  = {}", 1.0f64 / 0.0);
    println!("1.0 / -0.0  = {}", 1.0f64 / -0.0);
}
