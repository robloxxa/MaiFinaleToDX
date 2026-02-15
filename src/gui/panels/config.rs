use crate::config::Config;
use crate::runtime::ModuleRuntime;
use eframe::egui;
use log::info;

pub struct ConfigEditor {
    config_text: String,
    status_message: Option<(String, bool)>,
}

impl ConfigEditor {
    pub fn new(config: &Config) -> Self {
        let config_text = toml_edit::ser::to_string_pretty(config).unwrap_or_default();
        Self {
            config_text,
            status_message: None,
        }
    }

    pub fn reload_from(&mut self, config: &Config) {
        self.config_text = toml_edit::ser::to_string_pretty(config).unwrap_or_default();
    }
}

pub fn show(ui: &mut egui::Ui, editor: &mut ConfigEditor, runtime: &mut ModuleRuntime) {
    ui.heading("Configuration");
    ui.separator();

    if let Some((msg, is_ok)) = &editor.status_message {
        let color = if *is_ok {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.colored_label(color, msg.as_str());
        ui.add_space(5.0);
    }

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            match toml_edit::de::from_str::<Config>(&editor.config_text) {
                Ok(new_config) => {
                    match new_config.save("./config.toml") {
                        Ok(()) => {
                            runtime.update_config(new_config);
                            editor.status_message = Some(("Config saved".to_string(), true));
                            info!("Config saved from GUI");
                        }
                        Err(e) => {
                            editor.status_message =
                                Some((format!("Save failed: {}", e), false));
                        }
                    }
                }
                Err(e) => {
                    editor.status_message = Some((format!("Invalid TOML: {}", e), false));
                }
            }
        }

        if ui.button("Apply & Restart All").clicked() {
            match toml_edit::de::from_str::<Config>(&editor.config_text) {
                Ok(new_config) => {
                    if let Err(e) = new_config.save("./config.toml") {
                        editor.status_message = Some((format!("Save failed: {}", e), false));
                    } else {
                        runtime.update_config(new_config);
                        runtime.stop_all();
                        runtime.start_all();
                        editor.status_message =
                            Some(("Config applied, modules restarted".to_string(), true));
                        info!("Config applied and modules restarted from GUI");
                    }
                }
                Err(e) => {
                    editor.status_message = Some((format!("Invalid TOML: {}", e), false));
                }
            }
        }

        if ui.button("Reload from file").clicked() {
            editor.reload_from(runtime.config());
            editor.status_message = Some(("Reloaded from current config".to_string(), true));
        }
    });

    ui.add_space(10.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut editor.config_text)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(30),
            );
        });
}
