// 합계(reduction)를 세 가지 방식으로 구현합니다.
// 입력: u32 배열. 출력: 모든 원소의 합계(atomic<u32>)

@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> result: atomic<u32>;

// ---- v1: 모든 스레드가 전역 atomic 변수에 더합니다 ----
// 1,677만 개 스레드가 변수 하나를 경쟁합니다(5장 false sharing의 극단적인 형태)
@compute @workgroup_size(256)
fn reduce_atomic(
    @builtin(workgroup_id) wgid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let group = wgid.y * 256u + wgid.x; // 2차원 디스패치를 1차원 인덱스로 되돌립니다
    let i = group * 256u + lid.x;
    if (i < arrayLength(&input)) {
        atomicAdd(&result, input[i]);
    }
}

// ---- v2: 워크그룹 내부에서 공유 메모리 트리로 합친 뒤 대표 스레드 하나만 atomic에 더합니다 ----
var<workgroup> partial: array<u32, 256>;

@compute @workgroup_size(256)
fn reduce_shared(
    @builtin(workgroup_id) wgid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let group = wgid.y * 256u + wgid.x;
    let i = group * 256u + lid.x;
    var v = 0u;
    if (i < arrayLength(&input)) {
        v = input[i];
    }
    partial[lid.x] = v;
    workgroupBarrier();
    // 256 -> 128 -> 64 -> ... -> 1처럼 절반씩 합칩니다
    var stride = 128u;
    while (stride > 0u) {
        if (lid.x < stride) {
            partial[lid.x] += partial[lid.x + stride];
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (lid.x == 0u) {
        atomicAdd(&result, partial[0]);
    }
}

// ---- v3: 각 스레드가 먼저 레지스터에서 64개 원소를 합친 뒤 트리로 축약합니다 ----
@compute @workgroup_size(256)
fn reduce_multi(
    @builtin(workgroup_id) wgid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let n = arrayLength(&input);
    let threads = 1024u * 256u; // 전체 스레드 수
    var sum = 0u;
    // 그리드 스트라이드 루프: 각 스레드가 일정 간격으로 건너뛰며 전체를 처리합니다
    var i = wgid.x * 256u + lid.x;
    while (i < n) {
        sum += input[i];
        i += threads;
    }
    partial[lid.x] = sum;
    workgroupBarrier();
    var stride = 128u;
    while (stride > 0u) {
        if (lid.x < stride) {
            partial[lid.x] += partial[lid.x + stride];
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (lid.x == 0u) {
        atomicAdd(&result, partial[0]);
    }
}

