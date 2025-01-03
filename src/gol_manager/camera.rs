use glam::{vec2, Mat3, Vec2};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    wheel: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            wheel: 0.0,
        }
    }
    pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW => {
                        self.is_up_pressed = pressed;
                        true
                    }
                    KeyCode::KeyS => {
                        self.is_down_pressed = pressed;
                        true
                    }
                    KeyCode::KeyA => {
                        self.is_left_pressed = pressed;
                        true
                    }
                    KeyCode::KeyD => {
                        self.is_right_pressed = pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.wheel = *y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.wheel = pos.y as f32;
                    }
                }
                true
            }
            _ => false,
        }
    }
    pub fn update_camera(&mut self, camera: &mut Camera) {
        let mut movement = Vec2::ZERO;
        if self.is_up_pressed {
            movement.y += 1.0;
        }
        if self.is_down_pressed {
            movement.y -= 1.0;
        }
        if self.is_left_pressed {
            movement.x -= 1.0;
        }
        if self.is_right_pressed {
            movement.x += 1.0;
        }
        camera.zoom *= 1.0 - self.wheel * 0.04;
        camera.zoom = camera.zoom.clamp(0.1, 100.0);
        camera.position += movement.normalize_or_zero() * self.speed * camera.zoom;
        self.wheel = 0.0;
    }
}

pub struct Camera {
    pub position: Vec2,
    pub rotation: f32,
    pub zoom: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    pub(crate) fn get_matrix(&self) -> Mat3 {
        let view = Mat3::from_scale_angle_translation(
            vec2(self.zoom, self.zoom),
            self.rotation,
            self.position,
        )
        .inverse();
        let projection = Mat3::from_scale(vec2(1.0 / self.aspect_ratio, 1.0));
        projection * view
    }

    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            zoom: 1.0,
            aspect_ratio,
        }
    }
}