use eframe::egui;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const MAX_LOG_LINES: usize = 1000;

#[derive(Clone)]
pub struct LogBuffer {
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES))),
        }
    }

    pub fn push(&self, line: String) {
        let mut lines = self.lines.lock().unwrap();
        if lines.len() >= MAX_LOG_LINES {
            lines.pop_front();
        }
        lines.push_back(line);
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().iter().cloned().collect()
    }
}

pub fn show(ui: &mut egui::Ui, log_buffer: &LogBuffer) {
    ui.heading("Log");
    ui.separator();

    let lines = log_buffer.get_lines();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &lines {
                let color = if line.contains("ERROR") {
                    egui::Color32::RED
                } else if line.contains("WARN") {
                    egui::Color32::YELLOW
                } else if line.contains("DEBUG") {
                    egui::Color32::from_gray(150)
                } else {
                    egui::Color32::from_gray(220)
                };
                ui.colored_label(color, egui::RichText::new(line).monospace().size(11.0));
            }
        });
}
