use crate::{gui::components::module_header, runtime::{ModuleName, ModuleRuntime}, state::SharedState};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, runtime: &mut ModuleRuntime) {
    module_header::show(ui, runtime, ModuleName::Reader);
    
    ui.separator();

    let reader_state = runtime.shared_state().reader.lock().unwrap();

    ui.horizontal(|ui| {
        ui.label("Status:");
        if reader_state.polling {
            ui.colored_label(egui::Color32::GREEN, "Polling");
        } else {
            ui.colored_label(egui::Color32::GRAY, "Idle");
        }
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Last Card ID:");
        match &reader_state.last_card_id {
            Some(id) => {
                ui.monospace(id);
            }
            None => {
                ui.colored_label(egui::Color32::GRAY, "None");
            }
        }
    });
}
