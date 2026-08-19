//! 24장: 청크 처리에서 동기화 방식을 바꿔 전체 시간을 비교합니다.
//! 실행: cd examples && cargo run --release -p ch24-overlap

use std::num::NonZeroU64;
use std::time::Instant;
use wgpu::util::DeviceExt;

const CHUNKS: usize = 16;
const CHUNK_ELEMS: usize = 1024 * 1024; // 청크 하나 = 4MB

struct Chunk {
    buf_in: wgpu::Buffer,
    buf_out: wgpu::Buffer,
    buf_read: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

fn main() {
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

    let module = device.create_shader_module(wgpu::include_wgsl!("work.wgsl"));
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
        entries: &[storage(0, true), storage(1, false)],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&layout),
        module: &module,
        entry_point: Some("work"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // 청크마다 입력·출력·읽기 버퍼 한 세트를 준비합니다
    let bytes = (CHUNK_ELEMS * 4) as u64;
    let chunks: Vec<Chunk> = (0..CHUNKS)
        .map(|c| {
            let data: Vec<u32> = (0..CHUNK_ELEMS as u32).map(|i| i ^ c as u32).collect();
            let buf_in = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let buf_out = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let buf_read = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: bytes,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buf_in.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: buf_out.as_entire_binding() },
                ],
            });
            Chunk { buf_in, buf_out, buf_read, bind_group }
        })
        .collect();

    let groups = (CHUNK_ELEMS / 256) as u32;
    let record = |c: &Chunk| {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &c.bind_group, &[]);
            pass.dispatch_workgroups(groups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&c.buf_out, 0, &c.buf_read, 0, bytes);
        encoder.finish()
    };
    let checksum = |c: &Chunk| -> u64 {
        let slice = c.buf_read.slice(..);
        let view = slice.get_mapped_range().unwrap();
        let words: &[u32] = bytemuck::cast_slice(&view);
        let sum = words.iter().fold(0u64, |a, &x| a.wrapping_add(x as u64));
        drop(view);
        c.buf_read.unmap();
        sum
    };

    // 셰이더 컴파일 같은 초기 비용을 워밍업으로 제거합니다
    queue.submit([record(&chunks[0])]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    // ---- (A) 청크마다 완전히 동기화합니다 ----
    let start = Instant::now();
    let mut sum_a = 0u64;
    for c in &chunks {
        queue.submit([record(c)]);
        let slice = c.buf_read.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("매핑에 실패했습니다"));
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap(); // 매번 GPU를 기다립니다
        sum_a = sum_a.wrapping_add(checksum(c));
    }
    println!("(A) 청크마다 동기화      : {:>9.3?}", start.elapsed());

    // ---- (B) 모두 제출한 뒤 한꺼번에 회수합니다 ----
    let start = Instant::now();
    for c in &chunks {
        queue.submit([record(c)]); // 기다리지 않고 다음 작업을 제출합니다
    }
    for c in &chunks {
        c.buf_read.slice(..).map_async(wgpu::MapMode::Read, |r| r.expect("매핑에 실패했습니다"));
    }
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap(); // 대기는 한 번만 합니다
    let mut sum_b = 0u64;
    for c in &chunks {
        sum_b = sum_b.wrapping_add(checksum(c));
    }
    println!("(B) 모두 제출→일괄 회수: {:>9.3?}", start.elapsed());

    assert_eq!(sum_a, sum_b);
    println!("검증: OK (checksum={sum_a})");
}
