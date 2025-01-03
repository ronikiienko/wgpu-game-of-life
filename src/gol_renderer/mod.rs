use crate::gol::GoL;
use glam::{Mat3, Mat4, Vec2};
use egui_wgpu::wgpu;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    view_proj: [[f32; 4]; 4],
    quad_transform: [[f32; 4]; 4],
}

impl Uniform {
    fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            quad_transform: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
    fn update(&mut self, view_proj: Mat4, quad_transform: Mat4) {
        self.view_proj = view_proj.to_cols_array_2d();
        self.quad_transform = quad_transform.to_cols_array_2d();
    }
}

pub struct GoLRenderer {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform: Uniform,
}

impl GoLRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                },
            ],
        });
        let camera_uniform = Uniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[camera_uniform]),
        });
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            multiview: None,
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    write_mask: wgpu::ColorWrites::ALL,
                    blend: Some(wgpu::BlendState::REPLACE),
                })],
            }),
            layout: Some(&pipeline_layout),
            cache: None,
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            primitive: wgpu::PrimitiveState {
                conservative: false,
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                compilation_options: Default::default(),
                entry_point: Some("vs_main"),
                module: &shader_module,
                buffers: &[],
            },
        });

        Self {
            bind_group_layout,
            pipeline,
            uniform_buffer: camera_buffer,
            uniform: camera_uniform,
        }
    }

    /// To allow navigation and scrolling and dimension flexibility, i create a quad to which i render game of life
    /// Here you can pass transform matrix to move and scale the quad
    /// By default quad is at origin and has radius 1 (-1 to 1 in x and y)
    pub fn rerender(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        gol: &GoL,
        target_view: &wgpu::TextureView,
        view_proj: Mat3,
        quad_transform: Mat3,
    ) {
        self.uniform.update(Mat4::from_mat3(view_proj), Mat4::from_mat3(quad_transform));
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniform]));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        gol.get_current_view(),
                    ),
                },
            ],
            label: None,
            layout: &self.bind_group_layout,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                timestamp_writes: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, Some(&bind_group), &[]);
            render_pass.draw(0..6, 0..1);
        }
    }

    pub fn ndc_to_gol_uv(ndc: Vec2, view_proj: Mat3, quad_transform: Mat3) -> Vec2 {
        // Since quad to which we render is full-ndc, inverting transformations done in shader is enough
        let view_proj_inv = view_proj.inverse();
        let quad_transform_inv = quad_transform.inverse();
        let ndc_3d = Vec2::new(ndc.x, ndc.y).extend(1.0);
        let ndc_transformed = quad_transform_inv * view_proj_inv * ndc_3d;
        let mut uv = ndc_transformed.truncate() * 0.5 + Vec2::new(0.5, 0.5);
        uv.y = 1.0 - uv.y;
        uv
    }
}
