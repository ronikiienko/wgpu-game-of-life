use std::time::Duration;
use rand::Rng;

pub struct GameOfLifeFrag {
    tex_a: wgpu::Texture,
    tex_b: wgpu::Texture,
    tex_a_view: wgpu::TextureView,
    tex_b_view: wgpu::TextureView,
    read_from_a: bool,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}
impl GameOfLifeFrag {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, interval: Duration) -> Self {
        let texture_format = wgpu::TextureFormat::R8Uint;
        let descriptor = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            label: None,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            format: texture_format,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            view_formats: &[],
            sample_count: 1,
        };
        let view_descriptor: wgpu::TextureViewDescriptor = Default::default();
        let tex_a = device.create_texture(&descriptor);
        let tex_b = device.create_texture(&descriptor);
        let tex_a_view = tex_a.create_view(&view_descriptor);
        let tex_b_view = tex_b.create_view(&view_descriptor);

        let mut rng = rand::thread_rng();
        let initial_state: Vec<u8> = (0..width * height).map(|_| {
            let num: u32 = rng.gen_range(0..10);
            if num == 0 {
                return 1;
            } else {
                return 0;
            }
        }).collect();
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex_a,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d::ZERO,
            },
            &initial_state,
            wgpu::ImageDataLayout {
                rows_per_image: Some(height),
                bytes_per_row: Some(width),
                offset: 0,
            },
            descriptor.size,
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Game of Life Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
            ],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Game of Life Pipeline Layout"),
            push_constant_ranges: &[],
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Game of Life Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: Default::default(),
                entry_point: Some("vs_main"),
                module: &shader_module,
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                conservative: false,
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
            },
            multiview: None,
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            depth_stencil: None,
            cache: None,
        });

        Self {
            tex_a,
            tex_b,
            tex_a_view,
            tex_b_view,
            read_from_a: true,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> &wgpu::TextureView {
        let read_from_view = if self.read_from_a {
            &self.tex_a_view
        } else {
            &self.tex_b_view
        };
        let write_to_view = if self.read_from_a {
            &self.tex_b_view
        } else {
            &self.tex_a_view
        };
        if self.last_update.elapsed() < self.interval {
            return read_from_view;
        }
        self.last_update = std::time::Instant::now();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(read_from_view),
                },
            ],
            label: Some("Game of Life Bind Group"),
            layout: &self.bind_group_layout,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game of Life Render Pass"),
                occlusion_query_set: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: write_to_view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        self.read_from_a = !self.read_from_a;
        read_from_view
    }
}
