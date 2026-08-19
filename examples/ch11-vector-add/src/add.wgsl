// 벡터 덧셈: c[i] = a[i] + b[i]
// WGSL(WebGPU Shading Language)로 작성한 컴퓨트 셰이더입니다

// 바인드 그룹 0의 각 binding에 버퍼를 연결합니다
@group(0) @binding(0)
var<storage, read> a: array<f32>;

@group(0) @binding(1)
var<storage, read> b: array<f32>;

@group(0) @binding(2)
var<storage, read_write> c: array<f32>;

// 워크그룹 하나당 64개 스레드를 사용합니다.
// 이 함수는 원소 하나당 한 번씩 병렬로 호출됩니다
@compute @workgroup_size(64)
fn add(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    // 원소 수가 64의 배수가 아니면 범위를 벗어난 스레드는 아무 작업도 하지 않습니다
    if (i >= arrayLength(&a)) {
        return;
    }
    c[i] = a[i] + b[i];
}
