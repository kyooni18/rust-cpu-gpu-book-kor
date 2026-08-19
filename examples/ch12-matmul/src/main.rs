//! 12장: 같은 행렬 곱을 CPU 세 가지 방식과 GPU 세 가지 방식으로 계산합니다.
//! 실행: cargo run --release -p ch12-matmul [-- n]
//! n은 32의 배수입니다(기본값: 1024)

use rayon::prelude::*;
use std::num::NonZeroU64;
use std::time::Instant;
use wgpu::util::DeviceExt;

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// GFLOP/s를 계산합니다(행렬 곱의 연산 수는 2n^3입니다)
fn gflops(n: usize, secs: f64) -> f64 {
    (2.0 * (n as f64).powi(3)) / secs / 1e9
}

// ---- CPU 버전 ----

/// 단순한 3중 루프(ijk 순서). B를 열 방향으로 읽기 때문에 캐시에 불리합니다
fn matmul_naive(a: &[f32], b: &[f32], c: &mut [f32], n: usize) {
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a[i * n + k] * b[k * n + j];
            }
            c[i * n + j] = sum;
        }
    }
}

/// 루프 순서를 ikj로 바꾼 버전입니다. 모든 접근이 행 방향(연속)으로 바뀝니다
fn matmul_ikj(a: &[f32], b: &[f32], c: &mut [f32], n: usize) {
    c.fill(0.0);
    for i in 0..n {
        for k in 0..n {
            let aik = a[i * n + k];
            let b_row = &b[k * n..k * n + n];
            let c_row = &mut c[i * n..i * n + n];
            for j in 0..n {
                c_row[j] += aik * b_row[j];
            }
        }
    }
}

/// ikj 버전을 행 단위로 병렬화합니다
fn matmul_par(a: &[f32], b: &[f32], c: &mut [f32], n: usize) {
    c.par_chunks_mut(n).enumerate().for_each(|(i, c_row)| {
        c_row.fill(0.0);
        for k in 0..n {
            let aik = a[i * n + k];
            let b_row = &b[k * n..k * n + n];
            for j in 0..n {
                c_row[j] += aik * b_row[j];
            }
        }
    });
}

// ---- GPU 버전 ----

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group: wgpu::BindGroup,
    pipeline_naive: wgpu::ComputePipeline,
    pipeline_tiled: wgpu::ComputePipeline,
    pipeline_blocked: wgpu::ComputePipeline,
    buf_c: wgpu::Buffer,
    buf_read: wgpu::Buffer,
    n: usize,
}

impl Gpu {
    fn new(a: &[f32], b: &[f32], n: usize) -> Self {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("GPU를 찾을 수 없습니다");
        println!("GPU: {}", adapter.get_info().name);
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            }))
            .expect("Device 생성에 실패했습니다");

        let module = device.create_shader_module(wgpu::include_wgsl!("matmul.wgsl"));

        let buf_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("a"),
            contents: bytemuck::cast_slice(a),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let buf_b = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("b"),
            contents: bytemuck::cast_slice(b),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let buf_c = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("c"),
            size: (n * n * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let buf_read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (n * n * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let params = [u32::try_from(n).expect("n을 u32로 표현할 수 없습니다"), 0, 0, 0];
        let buf_params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("params"),
            contents: bytemuck::cast_slice(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                storage(0, true),
                storage(1, true),
                storage(2, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: Some(NonZeroU64::new(16).unwrap()),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_c.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_params.as_entire_binding(),
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let make_pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        Self {
            pipeline_naive: make_pipeline("matmul_naive"),
            pipeline_tiled: make_pipeline("matmul_tiled"),
            pipeline_blocked: make_pipeline("matmul_blocked"),
            device,
            queue,
            bind_group,
            buf_c,
            buf_read,
            n,
        }
    }

    /// 디스패치한 뒤 완료될 때까지 기다립니다(계산만 수행하고 결과는 읽지 않습니다).
    /// tile은 워크그룹 하나가 담당하는 C 블록의 한 변 길이입니다
    fn dispatch(&self, pipeline: &wgpu::ComputePipeline, tile: usize) {
        let groups = (self.n / tile) as u32;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(groups, groups, 1);
        }
        self.queue.submit([encoder.finish()]);
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
    }

    /// 결과를 CPU 쪽으로 읽어옵니다
    fn read_c(&self) -> Vec<f32> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&self.buf_c, 0, &self.buf_read, 0, self.buf_c.size());
        self.queue.submit([encoder.finish()]);
        let slice = self.buf_read.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| {
            r.expect("버퍼 매핑에 실패했습니다");
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        let data = slice.get_mapped_range().unwrap();
        let result = bytemuck::allocation::pod_collect_to_vec(&data);
        drop(data);
        self.buf_read.unmap();
        result
    }
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024);
    assert!(n > 0 && n % 32 == 0, "n은 32의 배수인 양수여야 합니다");
    // 4096^2 * 4B = 64MB. WebGPU의 storage 버퍼 기본 상한(128MiB) 안에 들어오게 제한합니다
    assert!(n <= 4096, "n은 4096 이하여야 합니다");
    println!("n = {n} ({}MB × 3)\n", n * n * 4 / 1024 / 1024);

    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let a: Vec<f32> = (0..n * n)
        .map(|_| (xorshift(&mut state) % 1000) as f32 / 1000.0)
        .collect();
    let b: Vec<f32> = (0..n * n)
        .map(|_| (xorshift(&mut state) % 1000) as f32 / 1000.0)
        .collect();
    let mut c = vec![0.0f32; n * n];

    // ---- CPU ----
    let start = Instant::now();
    matmul_naive(&a, &b, &mut c, n);
    let t = start.elapsed();
    println!("CPU naive(ijk)   : {t:>9.3?} ({:6.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    let reference = c.clone();

    let start = Instant::now();
    matmul_ikj(&a, &b, &mut c, n);
    let t = start.elapsed();
    println!("CPU ikj          : {t:>9.3?} ({:6.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    check("ikj", &reference, &c);

    let start = Instant::now();
    matmul_par(&a, &b, &mut c, n);
    let t = start.elapsed();
    println!("CPU ikj + rayon  : {t:>9.3?} ({:6.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    check("par", &reference, &c);

    // ---- GPU ----
    let gpu = Gpu::new(&a, &b, n);

    // 첫 실행에는 셰이더 컴파일 같은 초기 비용이 포함되므로 워밍업으로 버립니다
    gpu.dispatch(&gpu.pipeline_naive, 16);

    let start = Instant::now();
    gpu.dispatch(&gpu.pipeline_naive, 16);
    let t = start.elapsed();
    println!("GPU naive        : {t:>9.3?} ({:6.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    check("gpu naive", &reference, &gpu.read_c());

    gpu.dispatch(&gpu.pipeline_tiled, 16);
    let start = Instant::now();
    gpu.dispatch(&gpu.pipeline_tiled, 16);
    let t = start.elapsed();
    println!("GPU tiled        : {t:>9.3?} ({:6.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    check("gpu tiled", &reference, &gpu.read_c());

    gpu.dispatch(&gpu.pipeline_blocked, 32);
    let start = Instant::now();
    gpu.dispatch(&gpu.pipeline_blocked, 32);
    let t = start.elapsed();
    println!("GPU blocked      : {t:>9.3?} ({:6.1} GFLOP/s)", gflops(n, t.as_secs_f64()));
    check("gpu blocked", &reference, &gpu.read_c());

    // 전송 포함 시간: 데이터 전달 → 계산 → 결과 읽기를 모두 묶어서 측정합니다
    let start = Instant::now();
    let gpu2 = Gpu::new(&a, &b, n);
    gpu2.dispatch(&gpu2.pipeline_blocked, 32);
    let c_gpu = gpu2.read_c();
    let t = start.elapsed();
    println!("GPU blocked(초기화+전송 포함): {t:>9.3?}");
    check("gpu total", &reference, &c_gpu);
}

/// 부동소수점 반올림 순서 차이를 허용하면서 결과를 비교합니다
fn check(name: &str, reference: &[f32], result: &[f32]) {
    assert_eq!(reference.len(), result.len(), "[{name}] 길이가 일치하지 않습니다");
    let mut ok = true;
    for (i, (&x, &y)) in reference.iter().zip(result).enumerate() {
        let tol = 1e-3 + 1e-4 * x.abs();
        if !y.is_finite() || (x - y).abs() > tol {
            println!("  [{name}] 검증 실패: index {i}: 기대값 {x}, 실제값 {y}");
            ok = false;
            break;
        }
    }
    let _ = ok;
}
