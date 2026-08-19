// 각 원소에 어느 정도 무거운 계산을 적용합니다(해시 연산 반복)

@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(256)
fn work(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&input)) {
        return;
    }
    var x = input[i];
    // 전송 비용만 비교하는 실험이 되지 않도록 적당한 계산 부하를 넣습니다
    for (var k = 0u; k < 64u; k = k + 1u) {
        x = x ^ (x << 13u);
        x = x ^ (x >> 17u);
        x = x ^ (x << 5u);
    }
    output[i] = x;
}
