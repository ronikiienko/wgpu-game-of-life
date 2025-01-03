use egui::{Align2, Context};
use crate::{GoLSpeed, GoLState};

pub fn add_gui(ui: &Context, fps_text: &str, gol_state: &mut GoLState) {
    egui::Window::new("Foff")
        .default_open(true)
        .default_width(800.0)
        .resizable(true)
        .anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(&ui, |mut ui| {
            ui.add(egui::Label::new(fps_text));

            let pause_text = if gol_state.is_paused { "Resume" } else { "Pause" };

            let pause_button_response = ui.add(egui::Button::new(pause_text)).on_hover_text("Pause/Resume the simulation");
            if pause_button_response.clicked() {
                gol_state.is_paused = !gol_state.is_paused;
            }
            ui.end_row();
        });
}