mod game_of_life;
mod patterns;
mod perf_monitor;
mod renderer;

use crate::game_of_life::GameOfLife;
use crate::patterns::{
    get_blinker, get_heavy_weight_spaceship, get_light_weight_spaceship, get_loaf,
    get_middle_weight_spaceship, get_penta_decathlon, get_toad,
};
use crate::perf_monitor::PerfMonitor;
use glam::{vec2, Mat3, Mat4, Vec2};
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};
use crate::renderer::Renderer;

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
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
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
        camera.zoom = camera.zoom.clamp(0.01, 10.0);
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
    fn get_matrix(&self) -> Mat3 {
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



struct State<'a> {
    window: &'a Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_controller: CameraController,
    game_of_life: GameOfLife,
    perf_monitor: PerfMonitor,
    renderer: Renderer
}
impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    memory_hints: Default::default(),
                    required_limits: wgpu::Limits {
                        max_texture_dimension_2d: 16384 * 2,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let mut camera = Camera::new(size.width as f32 / size.height as f32);
        let camera_controller = CameraController::new(0.05);


        let game_size = 250;
        let game_of_life = GameOfLife::new(&device, game_size, game_size);
        let state: Vec<u8> = (0..game_size * game_size).map(|i| {
            if i < game_size * game_size / 2 {
                0
            } else {
                1
            }
        }).collect();
        game_of_life.write_area(&queue, &state, 0, 0, game_size, game_size);

        let mut perf_monitor = PerfMonitor::new();
        perf_monitor.start("update");

        let renderer = Renderer::new(&device, surface_format);

        Self {
            surface,
            surface_config,
            queue,
            device,
            window,
            size,
            camera,
            camera_controller,
            game_of_life,
            perf_monitor,
            renderer
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width <= 0 || new_size.height <= 0 {
            return;
        }
        self.size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        self.game_of_life.update(&self.device, &self.queue);
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let updated_summary = self.perf_monitor.start_frame();
        if updated_summary {
            println!("{}", self.perf_monitor.get_summary());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.renderer.rerender(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.game_of_life,
            &view,
            self.camera.get_matrix(),
            Mat3::IDENTITY,
        );

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window.id() => {
                    if !state.input(event) {
                        // UPDATED!
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                state.window.request_redraw();
                                state.update();
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
