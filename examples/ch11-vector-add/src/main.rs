//! 11장: wgpu를 이용한 벡터 덧셈 예제입니다.
//! 실행: cargo run --release -p ch11-vector-add

use std::num::NonZeroU64;
use std::time::Instant;
use wgpu::util::DeviceExt;

fn main() {
    let n = 1_000_000usize;
    let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..n).map(|i| (i * 2) as f32).collect();

    // ---- 1. GPU 연결 ----
    // Instance(wgpu 전체 상태) → Adapter(물리 GPU) → Device(논리 장치) + Queue(명령 제출)
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("GPU를 찾을 수 없습니다");
    println!("GPU: {}", adapter.get_info().name);

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
    .expect("Device 생성에 실패했습니다");

    // ---- 2. 셰이더 컴파일 ----
    let module = device.create_shader_module(wgpu::include_wgsl!("add.wgsl"));

    // ---- 3. 버퍼 준비 ----
    // 입력 버퍼 두 개(VRAM에 있고 CPU에서 초기 데이터를 기록합니다)
    let buf_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("a"),
        contents: bytemuck::cast_slice(&a),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let buf_b = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("b"),
        contents: bytemuck::cast_slice(&b),
        usage: wgpu::BufferUsages::STORAGE,
    });
    // 출력 버퍼(VRAM)
    let buf_c = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("c"),
        size: (n * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    // CPU에서 읽기 위한 복사 대상 버퍼
    let buf_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (n * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // ---- 4. 바인드 그룹: 셰이더의 binding 번호와 버퍼를 연결합니다 ----
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            buffer_entry(0, true),  // a: 읽기 전용
            buffer_entry(1, true),  // b: 읽기 전용
            buffer_entry(2, false), // c: 쓰기 가능
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
        ],
    });

    // ---- 5. 파이프라인: 셰이더와 레이아웃을 실행 가능한 상태로 만듭니다 ----
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&layout),
        module: &module,
        entry_point: Some("add"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // ---- 6. 명령 기록과 제출 ----
    let start = Instant::now();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        // 100만 원소 ÷ 워크그룹 크기 64 = 15,625개 워크그룹을 실행합니다
        pass.dispatch_workgroups(n.div_ceil(64) as u32, 1, 1);
    }
    // 결과를 읽기용 버퍼로 복사하는 명령도 기록합니다
    encoder.copy_buffer_to_buffer(&buf_c, 0, &buf_read, 0, buf_c.size());
    queue.submit([encoder.finish()]);

    // ---- 7. 결과 읽기 ----
    let slice = buf_read.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| {
        r.expect("버퍼 매핑에 실패했습니다");
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = slice.get_mapped_range().unwrap();
    let c: Vec<f32> = bytemuck::allocation::pod_collect_to_vec(&data);
    println!("GPU 실행+읽기: {:?}", start.elapsed());

    // ---- 8. CPU와 비교하고 결과를 검증합니다 ----
    let start = Instant::now();
    let c_cpu: Vec<f32> = a.iter().zip(&b).map(|(x, y)| x + y).collect();
    println!("CPU(1코어)   : {:?}", start.elapsed());

    let ok = c == c_cpu;
    println!("검증: {} (c[10] = {})", if ok { "OK" } else { "NG" }, c[10]);
}

// BindGroupLayoutEntry의 반복 코드를 함수로 묶습니다
fn buffer_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            min_binding_size: Some(NonZeroU64::new(4).unwrap()),
            has_dynamic_offset: false,
        },
        count: None,
    }
}
