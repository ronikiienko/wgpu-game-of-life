use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PerfMonitor {
    clocks: HashMap<String, Instant>,
    sample_size: i32,
    current_sample: i32,
    ms_per_frames: HashMap<String, f32>
}

impl PerfMonitor {
    pub fn new() -> Self {
        PerfMonitor {
            clocks: HashMap::new(),
            sample_size: 60,
            current_sample: 0,
            ms_per_frames: HashMap::new()
        }
    }

    pub fn start_frame(&mut self) -> bool {
        if self.current_sample >= self.sample_size {
            let ms_per_frame: HashMap<_, _> = self.clocks.iter().map(|(label, instant)| {
                let elapsed = instant.elapsed();
                let ms_per_frame = ((elapsed.as_micros() as f64) / 1000.0) / self.sample_size as f64;
                (label.clone(), ms_per_frame as f32)
            }).collect();
            self.clocks.iter_mut().for_each(|(_, instant)| *instant = Instant::now());
            self.ms_per_frames = ms_per_frame;
            self.current_sample = 0;
            true
        } else {
            self.current_sample += 1;
            false
        }
    }

    pub fn start(&mut self, label: &str) {
        self.clocks.insert(label.to_string(), Instant::now());
    }

    pub fn end(&mut self, label: &str) {
        self.clocks.remove(label);
    }

    pub fn get_summary(&self) -> String {
        self.ms_per_frames.iter().fold("".to_string(), |acc, (label, ms_per_frame)| {
            format!("{}: {:.1} fps\n", &label, 1000.0 / ms_per_frame)
        })
    }

    // pub fn get_fps(&self, label: &str) -> Option<f32> {
    //
    // }
}