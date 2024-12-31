use std::time::Duration;

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
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture_format = wgpu::TextureFormat::R8Uint;
        let descriptor = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            label: None,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Game of Life Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                count: None,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
            }],
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

    fn get_read_view(&self) -> &wgpu::TextureView {
        if self.read_from_a {
            &self.tex_a_view
        } else {
            &self.tex_b_view
        }
    }
    fn get_read_texture(&self) -> &wgpu::Texture {
        if self.read_from_a {
            &self.tex_a
        } else {
            &self.tex_b
        }
    }
    fn get_write_view(&self) -> &wgpu::TextureView {
        if self.read_from_a {
            &self.tex_b_view
        } else {
            &self.tex_a_view
        }
    }
    fn get_write_texture(&self) -> &wgpu::Texture {
        if self.read_from_a {
            &self.tex_b
        } else {
            &self.tex_a
        }
    }

    /// Internally, the game of life simulation uses two textures to store the state of the cells.
    /// One texture is currently read from, while the other is written to.
    /// This function alternates between the two textures. You should update view that is used for rendering after each update.
    /// This is view to texture that uses R8Uint format, where 1 means alive and 0 means dead for each cell.
    pub fn get_current_view(&self) -> &wgpu::TextureView {
        self.get_read_view()
    }

    /// This function should not be called multiple times before passed encoder is submitted.
    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let read_from_view = self.get_read_view();
        let write_to_view = self.get_write_view();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(read_from_view),
            }],
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
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.tex_a.size().width, self.tex_a.size().height)
    }

    /// This method should NOT be called after update() is called and before passed encoder is submitted.
    pub fn write_area(
        &self,
        queue: &wgpu::Queue,
        data: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        if width * height != data.len() as u32 {
            panic!("Data size does not match the area size");
        }
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.get_read_texture(),
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
            },
            data,
            wgpu::ImageDataLayout {
                rows_per_image: Some(height),
                bytes_per_row: Some(width),
                offset: 0,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        )
    }

    pub async fn read_area(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        if width % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT != 0 {
            panic!(
                "Width must be a multiple of {}",
                wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            );
        }

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Read Area Buffer"),
            size: (width * height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Read Area Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: self.get_read_texture(),
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
            },
            wgpu::ImageCopyBuffer {
                layout: wgpu::ImageDataLayout {
                    bytes_per_row: Some(width),
                    rows_per_image: Some(height),
                    offset: 0,
                },
                buffer: &buffer,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let mut vec: Vec<u8>;

        {
            let buffer_slice = buffer.slice(..);

            let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                sender.send(result).unwrap()
            });
            device.poll(wgpu::Maintain::Wait);
            receiver.receive().await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            vec = data.to_vec();
        }

        vec
    }
}
