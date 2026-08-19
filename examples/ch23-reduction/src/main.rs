//! 23장: GPU reduction(합계)을 세 단계로 최적화합니다.
//! 실행: cd examples && cargo run --release -p ch23-reduction

use std::num::NonZeroU64;
use std::time::Instant;
use wgpu::util::DeviceExt;

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn main() {
    let n = 16 * 1024 * 1024usize; // 1,677만 원소
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let data: Vec<u32> = (0..n).map(|_| (xorshift(&mut state) & 0x7F) as u32).collect();

    // CPU에서 정답과 실행 시간을 구합니다
    let start = Instant::now();
    let expected: u64 = data.iter().map(|&x| x as u64).sum();
    let cpu_time = start.elapsed();
    println!("CPU(1코어)      : {cpu_time:>9.3?} (sum={expected})");

    // ---- GPU 준비 ----
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

    let module = device.create_shader_module(wgpu::include_wgsl!("reduce.wgsl"));
    let buf_in = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("input"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let buf_result = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("result"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
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
        entries: &[storage(0, true), storage(1, false)],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: buf_in.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: buf_result.as_entire_binding() },
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

    // (버전 이름, 엔트리 포인트, 디스패치 형태)
    let groups_2d = ((n / 256).div_ceil(256)) as u32; // 256x256=65,536개 그룹
    let variants: [(&str, &str, (u32, u32)); 3] = [
        ("v1 모두 atomic       ", "reduce_atomic", (256, groups_2d)),
        ("v2 공유 메모리 트리 ", "reduce_shared", (256, groups_2d)),
        ("v3 스레드당 64원소  ", "reduce_multi", (1024, 1)),
    ];

    for (name, entry, (gx, gy)) in variants {
        let pipeline = make_pipeline(entry);
        // 워밍업 한 번 + 측정 한 번
        for round in 0..2 {
            queue.write_buffer(&buf_result, 0, &[0u8; 4]); // 합계를 0으로 되돌립니다
            let start = Instant::now();
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(gx, gy, 1);
            }
            encoder.copy_buffer_to_buffer(&buf_result, 0, &buf_read, 0, 4);
            queue.submit([encoder.finish()]);
            let slice = buf_read.slice(..);
            slice.map_async(wgpu::MapMode::Read, |r| {
                r.expect("버퍼 매핑에 실패했습니다");
            });
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            let got = {
                let view = slice.get_mapped_range().unwrap();
                u32::from_ne_bytes(view[0..4].try_into().unwrap())
            };
            buf_read.unmap();
            if round == 1 {
                let ok = got as u64 == expected;
                println!(
                    "{name}: {:>9.3?} (sum={got}{})",
                    start.elapsed(),
                    if ok { "" } else { " 검증 실패!" }
                );
            }
        }
    }
}
