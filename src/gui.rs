use egui::{Align2, Context};
pub fn add_gui(ui: &Context, fps_text: &str) {
    egui::Window::new("Fuck off")
        .default_open(true)
        .default_width(800.0)
        .resizable(true)
        .anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(&ui, |mut ui| {
            // let ms_per_frame_opt = self.perf_monitor.get_ms_per_frame("update");
            // let fps_text = ms_per_frame_opt.map_or("Fps: NaN".to_string(), |ms_per_frame| {
            //     format!("Fps: {:.1}", 1000.0 / ms_per_frame)
            // });
            ui.add(egui::Label::new(fps_text));
            // ui.horizontal(|ui| {
            //     if ui.button("Pause").clicked() {
            //         self.gol_speed = GoLSpeed::Pause;
            //     }
            //     if ui.button("Slow").clicked() {
            //         self.gol_speed = GoLSpeed::Slow;
            //     }
            //     if ui.button("Normal").clicked() {
            //         self.gol_speed = GoLSpeed::Normal;
            //     }
            //     if ui.button("Fast").clicked() {
            //         self.gol_speed = GoLSpeed::Fast;
            //     }
            //     if ui.button("Fastest").clicked() {
            //         self.gol_speed = GoLSpeed::Fastest;
            //     }
            // });

            ui.end_row();

            // proto_scene.egui(ui);
        });
}