// 행렬 곱 C = A × B (n×n, 행 우선 저장)

@group(0) @binding(0)
var<storage, read> a: array<f32>;

@group(0) @binding(1)
var<storage, read> b: array<f32>;

@group(0) @binding(2)
var<storage, read_write> c: array<f32>;

struct Params {
    n: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(3)
var<uniform> params: Params;

// ---- 단순 버전: 스레드 하나가 C의 원소 하나를 계산합니다 ----
@compute @workgroup_size(16, 16)
fn matmul_naive(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = params.n;
    let row = gid.y;
    let col = gid.x;
    if (row >= n || col >= n) {
        return;
    }
    var sum = 0.0;
    for (var k = 0u; k < n; k = k + 1u) {
        sum = sum + a[row * n + k] * b[k * n + col];
    }
    c[row * n + col] = sum;
}

// ---- 블록 버전: 스레드 하나가 C의 4×4 블록을 계산합니다 ----
// 스레드 하나당 읽기(5개) 대비 곱셈-누산(16회) 비율을 높입니다
@compute @workgroup_size(8, 8)
fn matmul_blocked(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = params.n; // n은 32의 배수라고 가정합니다
    let row0 = gid.y * 4u;
    let col0 = gid.x * 4u;
    // 4×4개의 누산 값을 레지스터에 유지합니다(var는 0으로 초기화됩니다)
    var acc: array<vec4<f32>, 4>;
    for (var k = 0u; k < n; k = k + 1u) {
        // B의 행에서 연속한 원소 4개를 한꺼번에 읽습니다
        let base = k * n + col0;
        let vb = vec4<f32>(b[base], b[base + 1u], b[base + 2u], b[base + 3u]);
        for (var i = 0u; i < 4u; i = i + 1u) {
            let aik = a[(row0 + i) * n + k];
            acc[i] = acc[i] + aik * vb;
        }
    }
    for (var i = 0u; i < 4u; i = i + 1u) {
        let base = (row0 + i) * n + col0;
        c[base] = acc[i].x;
        c[base + 1u] = acc[i].y;
        c[base + 2u] = acc[i].z;
        c[base + 3u] = acc[i].w;
    }
}

// ---- 타일 버전: 워크그룹(16×16)이 공유 메모리에 타일을 올려 재사용합니다 ----
const TILE: u32 = 16u;

var<workgroup> tile_a: array<f32, 256>;
var<workgroup> tile_b: array<f32, 256>;

@compute @workgroup_size(16, 16)
fn matmul_tiled(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let n = params.n; // n은 16의 배수라고 가정합니다
    let row = gid.y;
    let col = gid.x;
    var sum = 0.0;
    let tiles = n / TILE;
    for (var t = 0u; t < tiles; t = t + 1u) {
        // 각 스레드가 타일 원소 하나씩을 공유 메모리로 옮깁니다
        tile_a[lid.y * TILE + lid.x] = a[row * n + (t * TILE + lid.x)];
        tile_b[lid.y * TILE + lid.x] = b[(t * TILE + lid.y) * n + col];
        // 워크그룹의 모든 스레드가 복사를 끝낼 때까지 기다립니다
        workgroupBarrier();
        // 타일 안의 16개 원소에 대한 곱셈-누산은 모두 공유 메모리에서 읽습니다
        for (var k = 0u; k < TILE; k = k + 1u) {
            sum = sum + tile_a[lid.y * TILE + k] * tile_b[k * TILE + lid.x];
        }
        workgroupBarrier();
    }
    c[row * n + col] = sum;
}
