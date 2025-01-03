use glam::{vec2, Mat3, Vec2};
use winit::event::{MouseButton, WindowEvent};
use std::sync::Arc;
use egui_wgpu::wgpu;
use crate::gol::GoL;
use crate::gol_renderer::GoLRenderer;

pub struct GoLDrawing {
    mouse_position: Option<Vec2>,
}

impl GoLDrawing {
    pub fn new() -> Self {
        Self {
            mouse_position: None,
        }
    }
    pub fn handle_input(
        &mut self,
        event: &WindowEvent,
        window: Arc<winit::window::Window>,
        gol: &GoL,
        gol_view_proj: Mat3,
        gol_quad_transform: Mat3,
        queue: &wgpu::Queue,
    ) -> bool {
        match event {
            WindowEvent::MouseInput { button, state, .. } => {
                if state.is_pressed() {
                    if let Some(mouse_position) = self.mouse_position {
                        let mut ndc = mouse_position / vec2(window.inner_size().width as f32, window.inner_size().height as f32) * 2.0 - vec2(1.0, 1.0);
                        ndc.y = -ndc.y;
                        let uv = GoLRenderer::ndc_to_gol_uv(ndc, gol_view_proj, gol_quad_transform);
                        if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
                            return true;
                        }
                        let pixel_gol_position = uv * vec2(gol.get_size().0 as f32, gol.get_size().1 as f32);

                        let new_value = if *button == MouseButton::Left {
                            1
                        } else if *button == MouseButton::Right {
                            0
                        } else {
                            return false;
                        };
                        gol.write_area(queue, &[new_value], pixel_gol_position.x as u32, pixel_gol_position.y as u32, 1, 1);
                        return true;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Some(vec2(position.x as f32, position.y as f32));
                return false;
            }
            _ => {}
        }
        false
    }
}