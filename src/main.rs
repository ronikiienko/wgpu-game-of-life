mod camera;
mod drawing;
mod gol;
mod gol_renderer;
mod gui_adder;
mod gui_renderer;
mod patterns;
mod perf_monitor;

use crate::gol::GoL;
use crate::gol_renderer::GoLRenderer;
use crate::gui_adder::add_gui;
use crate::gui_renderer::EguiRenderer;
use crate::patterns::{
    get_blinker, get_heavy_weight_spaceship, get_light_weight_spaceship, get_loaf,
    get_middle_weight_spaceship, get_penta_decathlon, get_toad,
};
use crate::perf_monitor::PerfMonitor;
use camera::{Camera, CameraController};
use drawing::GoLDrawing;
use egui::Align2;
use egui_wgpu::wgpu;
use glam::{vec2, Mat3, Mat4, UVec2, Vec2};
use std::sync::Arc;
use std::task::Context;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

pub struct GoLKeyboardController {}
impl GoLKeyboardController {
    pub fn handle_input(&self, event: &WindowEvent, gol_config: &mut GoLConfig) -> bool {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if let PhysicalKey::Code(keycode) = event.physical_key {
                if event.state == ElementState::Pressed && !event.repeat {
                    return match keycode {
                        KeyCode::Space => {
                            gol_config.is_paused = !gol_config.is_paused;
                            true
                        }
                        KeyCode::Digit1 => {
                            gol_config.speed = GoLSpeed::Slow;
                            true
                        }
                        KeyCode::Digit2 => {
                            gol_config.speed = GoLSpeed::Normal;
                            true
                        }
                        KeyCode::Digit3 => {
                            gol_config.speed = GoLSpeed::Fast;
                            true
                        }
                        KeyCode::Digit4 => {
                            gol_config.speed = GoLSpeed::Fastest;
                            true
                        }
                        _ => false,
                    };
                }
                return false;
            }
        }
        false
    }
    pub fn new() -> Self {
        Self {}
    }
}

enum GoLSpeed {
    Slow,
    Normal,
    Fast,
    Fastest,
}
impl GoLSpeed {
    pub fn get_interval(&self) -> Duration {
        match self {
            GoLSpeed::Slow => Duration::from_millis(500),
            GoLSpeed::Normal => Duration::from_millis(100),
            GoLSpeed::Fast => Duration::from_millis(50),
            GoLSpeed::Fastest => Duration::from_millis(10),
        }
    }
}

struct GoLConfig {
    pub is_paused: bool,
    pub speed: GoLSpeed,
}

struct GoLState {
    config: GoLConfig,
    render_quad_transform: Mat3,
    gol: GoL,
    renderer: GoLRenderer,
    camera: Camera,
    camera_controller: CameraController,
    keyboard_controller: GoLKeyboardController,
    drawing: GoLDrawing,
    gui_renderer: EguiRenderer,
    perf_monitor: PerfMonitor,
}
impl GoLState {
    pub fn new(
        aspect_ratio: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: Arc<winit::window::Window>,
        render_target_format: wgpu::TextureFormat,
    ) -> Self {
        let mut camera = Camera::new(aspect_ratio);
        let camera_controller = CameraController::new(0.05);

        let game_width = 25000;
        let game_height = 25000;
        let gol = GoL::new(&device, game_width, game_height);
        let state: Vec<u8> = (0..game_width * game_height)
            .map(|i| {
                if i < game_width * game_height / 2 {
                    0
                } else {
                    1
                }
            })
            .collect();
        gol.write_area(&queue, &state, 0, 0, game_width, game_height);

        let renderer = GoLRenderer::new(&device, render_target_format);

        let baseline_size = 500;
        let gol_size_tuple = gol.get_size();
        let gol_size = vec2(gol_size_tuple.0 as f32, gol_size_tuple.1 as f32);
        let scale = vec2(
            gol_size.x / baseline_size as f32,
            gol_size.y / baseline_size as f32,
        );
        let render_quad_transform = Mat3::from_scale(scale);

        let gui_renderer = EguiRenderer::new(
            &device,
            render_target_format,
            None,
            1,
            false,
            window.clone(),
        );

        let mut perf_monitor = PerfMonitor::new();
        perf_monitor.start("update");

        Self {
            config: GoLConfig {
                is_paused: false,
                speed: GoLSpeed::Normal,
            },
            render_quad_transform,
            gol,
            renderer,
            camera,
            camera_controller,
            keyboard_controller: GoLKeyboardController::new(),
            drawing: GoLDrawing::new(),
            gui_renderer,
            perf_monitor,
        }
    }
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.camera_controller.update_camera(&mut self.camera);
        self.gol.update(device, queue);
    }
    pub fn handle_input(
        &mut self,
        event: &WindowEvent,
        window: Arc<winit::window::Window>,
        queue: &wgpu::Queue,
    ) -> bool {
        self.gui_renderer.handle_input(&window, event)
            || self.keyboard_controller.handle_input(event, &mut self.config)
            || self.camera_controller.handle_input(event)
            || self.drawing.handle_input(
                event,
                window.clone(),
                &self.gol,
                self.camera.get_matrix(),
                self.render_quad_transform,
                queue,
            )
    }
    pub fn handle_aspect_ratio_change(&mut self, new_aspect_ratio: f32) {
        self.camera.aspect_ratio = new_aspect_ratio;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &mut wgpu::Queue,
        target_view: &wgpu::TextureView,
        window: &winit::window::Window,
    ) {
        self.perf_monitor.start_frame();
        self.renderer.rerender(
            &device,
            &queue,
            encoder,
            &self.gol,
            &target_view,
            self.camera.get_matrix(),
            self.render_quad_transform,
        );

        self.gui_renderer.draw(
            device,
            queue,
            encoder,
            window,
            &target_view,
            egui_wgpu::ScreenDescriptor {
                size_in_pixels: [window.inner_size().width, window.inner_size().height],
                pixels_per_point: window.scale_factor() as f32,
            },
            |ui| {
                let ms_per_frame_opt = self.perf_monitor.get_ms_per_frame("update");
                let fps_text = ms_per_frame_opt.map_or("Fps: NaN".to_string(), |ms_per_frame| {
                    format!("Fps: {:.1}", 1000.0 / ms_per_frame)
                });
                add_gui(ui, &fps_text, &mut self.config);
            },
        );
    }
}

struct State {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    gol_state: GoLState,
}
impl State {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
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

        let gol_state = GoLState::new(
            size.width as f32 / size.height as f32,
            &device,
            &queue,
            window.clone(),
            surface_format,
        );

        Self {
            surface,
            surface_config,
            queue,
            device,
            window,
            size,
            gol_state,
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
        self.gol_state
            .handle_aspect_ratio_change(new_size.width as f32 / new_size.height as f32);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.gol_state.handle_input(event, self.window.clone(), &self.queue)
    }

    pub fn update(&mut self) {
        self.gol_state.update(&self.device, &self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.gol_state
            .render(&self.device, &mut encoder, &mut self.queue, &view, &self.window);

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default)]
pub struct App {
    state: Option<State>,
    window: Option<Arc<winit::window::Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = winit::window::Window::default_attributes();
        let window = event_loop.create_window(attributes).unwrap();

        let is_first_window_handle = self.window.is_none();
        let window_handle = Arc::new(window);
        self.window = Some(window_handle.clone());
        if is_first_window_handle {
            let state = pollster::block_on(State::new(window_handle.clone()));
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            panic!("Window or state not initialized");
        };

        if window_id != state.window.id() {
            return;
        }
        let consumed = state.input(&event);
        if consumed {
            return;
        }
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
            } => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                state.window.request_redraw();
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
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

fn main() {
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
