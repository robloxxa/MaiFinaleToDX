use eframe::egui;

use crate::{runtime::{ModuleName, ModuleRuntime}, state::ModuleStatus};


pub fn show(ui: &mut egui::Ui, runtime: &mut ModuleRuntime, name: ModuleName) {
    let status = runtime.module_status(name);
    
    ui.horizontal(|ui| {
        ui.heading(name.as_str());
        ui.horizontal(|ui| {
            match status {
                ModuleStatus::Stopped | ModuleStatus::Error => {
                    if ui.button("Start").clicked() {
                        runtime.start_module(name);
                    }
                }
                ModuleStatus::Running => {
                    if ui.button("Stop").clicked() {
                        runtime.stop_module(name);
                    }
                    if ui.button("Restart").clicked() {
                        runtime.restart_module(name);
                    }
                }
                ModuleStatus::Initializing => {
                    ui.add_enabled(false, egui::Button::new("..."));
                }
            }
        });
    });
        
    let (status_text, color) = match status {
            ModuleStatus::Stopped => ("Stopped", egui::Color32::GRAY),
            ModuleStatus::Initializing => ("Initializing...", egui::Color32::YELLOW),
            ModuleStatus::Running => ("Running", egui::Color32::GREEN),
            ModuleStatus::Error => ("Error", egui::Color32::RED),
    };
    
    ui.colored_label(color, format!("Status: {}", status_text));
}