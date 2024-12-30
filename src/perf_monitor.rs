use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PerfMonitor {
    clocks: HashMap<String, Instant>,
    sample_size: i32,
    current_sample: i32,
    summary: String
}

impl PerfMonitor {
    pub fn new() -> Self {
        PerfMonitor {
            clocks: HashMap::new(),
            sample_size: 60,
            current_sample: 0,
            summary: String::new()
        }
    }

    pub fn start_frame(&mut self) -> bool {
        if self.current_sample >= self.sample_size {
            self.summary = String::new();
            for (label, instant) in self.clocks.iter_mut() {
                let elapsed = instant.elapsed();
                let duration_per_frame = ((elapsed.as_micros() as f64) / 1000.0) / self.sample_size as f64;
                self.summary.push_str(&format!("{}: {}fps\n", label, 1000.0 / duration_per_frame));
                *instant = Instant::now();
            }
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

    pub fn get_summary(&self) -> &str {
        &self.summary
    }
}