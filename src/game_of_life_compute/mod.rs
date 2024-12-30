use rand::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;

pub struct GameOfLifeCompute {
    read_from_a: bool,
    buffer_a: wgpu::Buffer,
    buffer_b: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    last_update: std::time::Instant,
    interval: Duration,
    block_size: u32,
    work_group_count: (u32, u32, u32),
}

impl GameOfLifeCompute {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        interval: Duration,
    ) -> Self {
        let block_size = 8u32;
        if width % block_size != 0 || height % block_size != 0 {
            panic!("width and height must be divisible by {}", block_size);
        }
        let work_group_count = (width / block_size, height / block_size, 1);

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: None,
            size: (width * height * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer_a = device.create_buffer(&buffer_descriptor);
        let mut rng = rand::thread_rng();
        let values: Vec<u32> = (0..width * height)
            .map(|i| {
                if i > width {
                    return 0;
                }
                return 1;
                let num: u32 = rng.gen_range(0..10);
                if num == 0 {
                    return 1;
                } else {
                    return 0;
                }
            })
            .collect();
        queue.write_buffer(
            &buffer_a,
            0,
            bytemuck::cast_slice(&values),
        );
        let buffer_b = device.create_buffer(&buffer_descriptor);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[width, height]),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                },
            ],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            cache: None,
            compilation_options: Default::default(),
            entry_point: Some("main"),
            module: &shader_module,
        });

        Self {
            buffer_b,
            buffer_a,
            read_from_a: true,
            pipeline,
            bind_group_layout,
            uniform_buffer,
            last_update: Instant::now(),
            interval,
            block_size,
            work_group_count,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> &wgpu::Buffer {
        let read_from_buffer = if self.read_from_a {
            &self.buffer_a
        } else {
            &self.buffer_b
        };
        let write_to_buffer = if self.read_from_a {
            &self.buffer_b
        } else {
            &self.buffer_a
        };
        if self.last_update.elapsed() < self.interval {
            return read_from_buffer;
        }
        self.last_update = Instant::now();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: read_from_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: write_to_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                self.work_group_count.0,
                self.work_group_count.1,
                self.work_group_count.2,
            );
        }

        self.read_from_a = !self.read_from_a;
        write_to_buffer
    }
}
