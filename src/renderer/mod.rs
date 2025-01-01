use crate::game_of_life::GameOfLife;

struct Renderer {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,

}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {

    }
    pub fn rerender(&self, device: &wgpu::Device, encoder: &wgpu::CommandEncoder, gol: &GameOfLife, target: &wgpu::TextureView) {

    }
}