use std::time::Instant;
use bytemuck;
use wgpu::{PowerPreference::HighPerformance, util::DeviceExt};

fn cpu_workload(data: &mut [u32]) {
    for (index, val) in data.iter_mut().enumerate() {
        let mut x = index as u32;
        let mut hash = 0u32;
        for i in 0..300u32 {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            hash = hash.wrapping_add(x).wrapping_add(i);
        }
        *val = hash;
    }
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let num_elements = 16_000_000;
    println!("Dataset: {} elements", num_elements);

    let numbers: Vec<u32> = vec![0; num_elements];

    // CPU Computation
    println!("\nStarting CPU computation...");
    let mut cpu_data = numbers.clone();
    let cpu_start = Instant::now();
    cpu_workload(&mut cpu_data);
    let cpu_duration = cpu_start.elapsed();
    println!("CPU Time: {:.2?}", cpu_duration);

    // GPU Computation
    println!("\nStarting GPU computation...");
    let gpu_start = Instant::now();

    let slice_size = std::mem::size_of_val(&numbers[..]);
    let size = slice_size as wgpu::BufferAddress;

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions{power_preference: HighPerformance, ..Default::default()})
        .await
        .expect("Failed to find adapter");

    let info = adapter.get_info();
    println!("Using GPU: {} ({:?})", info.name, info.backend);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("Failed to create device");

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(&numbers),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    });

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: None,
        module: &shader,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);

        let workgroup_count = (num_elements as u32 + 255) / 256;
        cpass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);
    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::Wait);
    receiver.recv().unwrap().unwrap();

    let data = buffer_slice.get_mapped_range();
    let gpu_result: &[u32] = bytemuck::cast_slice(&data);

    let gpu_duration = gpu_start.elapsed();
    println!("GPU Time: {:.2?}", gpu_duration);

    // Exact verification
    assert_eq!(&cpu_data[..], gpu_result, "CPU and GPU results mismatch!");
    println!("\n✅ Verification Passed: Outputs are 100% identical!");
}