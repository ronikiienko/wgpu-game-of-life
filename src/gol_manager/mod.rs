mod gui_renderer;
mod gui_adder;
mod camera;

use std::sync::Arc;
use glam::{vec2, Mat3};
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::time::{Duration, Instant};
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::Instance;
use crate::drawing::GoLDrawing;
use crate::gol::GoL;
use crate::gol_renderer::GoLRenderer;
use gui_adder::add_gui;
use gui_renderer::EguiRenderer;
use crate::gol_manager::camera::{Camera, CameraController};
use crate::perf_monitor::PerfMonitor;

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
                            gol_config.speed = GoLSpeed::Slowest;
                            true
                        }
                        KeyCode::Digit2 => {
                            gol_config.speed = GoLSpeed::Slower;
                            true
                        }
                        KeyCode::Digit3 => {
                            gol_config.speed = GoLSpeed::Slow;
                            true
                        }
                        KeyCode::Digit4 => {
                            gol_config.speed = GoLSpeed::Normal;
                            true
                        }
                        KeyCode::Digit5 => {
                            gol_config.speed = GoLSpeed::Fast;
                            true
                        }
                        KeyCode::Digit6 => {
                            gol_config.speed = GoLSpeed::Faster;
                            true
                        }
                        KeyCode::Digit7 => {
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
    Slowest,
    Slower,
    Slow,
    Normal,
    Fast,
    Faster,
    Fastest,
}

impl GoLSpeed {
    pub fn get_interval(&self) -> Duration {
        match self {
            GoLSpeed::Slowest => Duration::from_millis(1000),
            GoLSpeed::Slower => Duration::from_millis(500),
            GoLSpeed::Slow => Duration::from_millis(250),
            GoLSpeed::Normal => Duration::from_millis(100),
            GoLSpeed::Fast => Duration::from_millis(30),
            GoLSpeed::Faster => Duration::from_millis(15),
            GoLSpeed::Fastest => Duration::from_millis(1),
        }
    }
}

pub struct GoLConfig {
    pub is_paused: bool,
    pub speed: GoLSpeed,
}

pub struct GoLManager {
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
    time_accumulator: Duration,
    last_update: Instant,
    max_ms_per_update: Duration,
}

impl GoLManager {
    pub fn new(
        aspect_ratio: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: Arc<winit::window::Window>,
        render_target_format: wgpu::TextureFormat,
    ) -> Self {
        let mut camera = Camera::new(aspect_ratio);
        let camera_controller = CameraController::new(0.05);

        let game_width = 32000;
        let game_height = 32000;
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
            time_accumulator: Duration::from_secs(0),
            last_update: Instant::now(),
            max_ms_per_update: Duration::from_millis(50),
        }
    }
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.camera_controller.update_camera(&mut self.camera);

        // don't update if last update took too long. This is to prevent snowballing updates.
        // For example, if simulation can't keep up update takes too long -> next update would take even longer (since last update took longer and more updates are queued up)
        if !self.config.is_paused && self.last_update.elapsed() < self.max_ms_per_update {
            self.time_accumulator += self.last_update.elapsed();
            self.last_update = Instant::now();

            while self.time_accumulator >= self.config.speed.get_interval() {
                self.time_accumulator -= self.config.speed.get_interval();
                self.gol.update(device, queue);
            }
        } else {
            self.last_update = Instant::now();
            self.time_accumulator = Duration::from_secs(0);
        }
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