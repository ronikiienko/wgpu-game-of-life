mod camera;
mod gol;
mod gol_renderer;
mod gui;
mod patterns;
mod perf_monitor;

use crate::gol::GoL;
use crate::gol_renderer::GoLRenderer;
use crate::gui::EguiRenderer;
use crate::patterns::{
    get_blinker, get_heavy_weight_spaceship, get_light_weight_spaceship, get_loaf,
    get_middle_weight_spaceship, get_penta_decathlon, get_toad,
};
use crate::perf_monitor::PerfMonitor;
use camera::{Camera, CameraController};
use egui::Align2;
use egui_wgpu::wgpu;
use glam::{vec2, Mat3, Mat4, Vec2};
use std::sync::Arc;
use std::task::Context;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

pub struct Drawing {}
impl Drawing {
    pub fn new() -> Self {
        Self {}
    }
    pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { button, state, .. } => {
                if *button == MouseButton::Left {
                    println!("Mouse button left {:?} {:?}", button, state);
                    return true
                }
            }
            _ => {}
        }
        false
    }
}

struct State {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_controller: CameraController,
    gol: GoL,
    perf_monitor: PerfMonitor,
    gol_renderer: GoLRenderer,
    gui_renderer: EguiRenderer,
    drawing: Drawing,
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

        let mut camera = Camera::new(size.width as f32 / size.height as f32);
        let camera_controller = CameraController::new(0.05);

        let game_width = 25000;
        let game_height = 25000;
        let game_of_life = GoL::new(&device, game_width, game_height);
        let state: Vec<u8> = (0..game_width * game_height)
            .map(|i| {
                if i < game_width * game_height / 2 {
                    0
                } else {
                    1
                }
            })
            .collect();
        game_of_life.write_area(&queue, &state, 0, 0, game_width, game_height);

        let mut perf_monitor = PerfMonitor::new();
        perf_monitor.start("update");

        let gol_renderer = GoLRenderer::new(&device, surface_format);
        let gui_renderer =
            EguiRenderer::new(&device, surface_format, None, 1, false, window.clone());

        let drawing = Drawing::new();

        Self {
            surface,
            surface_config,
            queue,
            device,
            window,
            size,
            camera,
            camera_controller,
            gol: game_of_life,
            perf_monitor,
            gol_renderer,
            gui_renderer,
            drawing
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
        self.gui_renderer.handle_input(&self.window, event)
            || self.camera_controller.handle_input(event)
            || self.drawing.handle_input(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.gol.update(&self.device, &self.queue);
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

        let baseline_size = 500;
        let gol_size_tuple = self.gol.get_size();
        let gol_size = vec2(gol_size_tuple.0 as f32, gol_size_tuple.1 as f32);
        let scale = vec2(
            gol_size.x / baseline_size as f32,
            gol_size.y / baseline_size as f32,
        );

        self.gol_renderer.rerender(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.gol,
            &view,
            self.camera.get_matrix(),
            Mat3::from_scale(scale),
        );

        self.gui_renderer.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.size.width, self.size.height],
                pixels_per_point: self.window.scale_factor() as f32,
            },
            |ui| {
                egui::Window::new("Streamline CFD")
                    // .vscroll(true)
                    .default_open(true)
                    .max_width(1000.0)
                    .max_height(800.0)
                    .default_width(800.0)
                    .resizable(true)
                    .anchor(Align2::LEFT_TOP, [0.0, 0.0])
                    .show(&ui, |mut ui| {
                        if ui.add(egui::Button::new("Click me")).clicked() {
                            println!("PRESSED")
                        }

                        ui.label("Slider");
                        // ui.add(egui::Slider::new(_, 0..=120).text("age"));
                        ui.end_row();

                        // proto_scene.egui(ui);
                    });
            },
        );

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
