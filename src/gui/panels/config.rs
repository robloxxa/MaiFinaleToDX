use crate::config::Config;
use crate::gui::components::circle_container;
use crate::gui::panels::Panel;
use crate::runtime::ModuleRuntime;
use eframe::egui;
use tracing::info;

pub struct ConfigEditor {
    pub config_text: String,
    pub dirty: bool,
    status_message: Option<(String, bool)>,
}

impl ConfigEditor {
    pub fn new(config: &Config) -> Self {
        let config_text = toml_edit::ser::to_string_pretty(config).unwrap_or_default();
        Self {
            config_text,
            dirty: false,
            status_message: None,
        }
    }

    pub fn reload_from(&mut self, config: &Config) {
        self.config_text = toml_edit::ser::to_string_pretty(config).unwrap_or_default();
        self.dirty = false;
    }
}

pub struct ConfigPanel<'a> {
    editor: &'a mut ConfigEditor,
    runtime: &'a mut ModuleRuntime,
}

impl<'a> ConfigPanel<'a> {
    pub fn new(editor: &'a mut ConfigEditor, runtime: &'a mut ModuleRuntime) -> Self {
        Self { editor, runtime }
    }
}

impl<'a> Panel for ConfigPanel<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");

        if let Some((msg, is_ok)) = &self.editor.status_message {
            let color = if *is_ok {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            };
            ui.colored_label(color, msg.as_str());
        }

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                match toml_edit::de::from_str::<Config>(&self.editor.config_text) {
                    Ok(new_config) => match new_config.save("./config.toml") {
                        Ok(()) => {
                            self.runtime.update_config(new_config);
                            self.editor.dirty = false;
                            self.editor.status_message =
                                Some(("Config saved".to_string(), true));
                            info!("Config saved from GUI");
                        }
                        Err(e) => {
                            self.editor.status_message =
                                Some((format!("Save failed: {}", e), false));
                        }
                    },
                    Err(e) => {
                        self.editor.status_message =
                            Some((format!("Invalid TOML: {}", e), false));
                    }
                }
            }

            if ui.button("Reload from file").clicked() {
                self.editor.reload_from(self.runtime.config());
                self.editor.status_message =
                    Some(("Reloaded from current config".to_string(), true));
            }
        });
    }

    fn left_body(&mut self, ui: &mut egui::Ui) {
        let mut inner = circle_container::inscribed_square(ui);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(&mut inner, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.editor.config_text)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(30),
                );
                if response.changed() {
                    self.editor.dirty = true;
                }
            });
    }
}
