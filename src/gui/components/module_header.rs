use eframe::egui;

use crate::{
    runtime::{ModuleName, ModuleRuntime},
    state::ModuleStatus,
};

pub enum ModuleHeaderAction {
    None,
    Start,
    Stop,
    Restart,
}

impl ModuleHeaderAction {
    pub fn apply(self, runtime: &mut ModuleRuntime, name: ModuleName) {
        match self {
            Self::None => {}
            Self::Start => runtime.start_module(name),
            Self::Stop => runtime.stop_module(name),
            Self::Restart => runtime.restart_module(name),
        }
    }
}


pub fn show_collapsible<R>(
    ui: &mut egui::Ui,
    name: ModuleName,
    status: ModuleStatus,
    add_body: impl FnOnce(&mut egui::Ui) -> R,
) -> (ModuleHeaderAction, Option<R>) {
    let id = ui.id().with(name.as_str());
    let mut action = ModuleHeaderAction::None;

    let (_, _, body_inner) =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.label(name.as_str());
                
                let (status_text, color) = match &status {
                    ModuleStatus::Stopped => ("Stopped", egui::Color32::GRAY),
                    ModuleStatus::Initializing => ("Initializing...", egui::Color32::YELLOW),
                    ModuleStatus::Running => ("Running", egui::Color32::GREEN),
                    ModuleStatus::Error(_) => ("Error", egui::Color32::RED),
                };

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match &status {
                        ModuleStatus::Stopped | ModuleStatus::Error(_) => {
                            if ui.button("Start").clicked() {
                                action = ModuleHeaderAction::Start;
                            }
                        }
                        ModuleStatus::Running => {
                            if ui.button("Restart").clicked() {
                                action = ModuleHeaderAction::Restart;
                            }
                            if ui.button("Stop").clicked() {
                                action = ModuleHeaderAction::Stop;
                            }
                        }
                        ModuleStatus::Initializing => {
                            ui.add_enabled(false, egui::Button::new("..."));
                        }
                    }
                    ui.colored_label(color, status_text);
                    if let ModuleStatus::Error(msg) = &status {
                        ui.colored_label(egui::Color32::RED, msg.as_ref());
                    }
                });
            })
            .body(|ui| add_body(ui));

    (action, body_inner.map(|r| r.inner))
}
