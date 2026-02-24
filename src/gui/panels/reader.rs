use crate::gui::components::{circle_container, module_header, port_combobox, ConfigWidgets};
use crate::gui::panels::Panel;
use crate::runtime::{ModuleName, ModuleRuntime};
use std::collections::HashSet;
use eframe::egui::{self, RichText};

pub struct Reader<'a> {
    runtime: &'a mut ModuleRuntime,
    pending_restarts: &'a mut HashSet<ModuleName>,
}

impl<'a> Reader<'a> {
    pub fn new(
        runtime: &'a mut ModuleRuntime,
        pending_restarts: &'a mut HashSet<ModuleName>,
    ) -> Self {
        Self { runtime, pending_restarts }
    }
}

impl<'a> Panel for Reader<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        module_header::show(ui, self.runtime, ModuleName::Reader);

        {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);

            ui.horizontal(|ui| {
                w.labeled_config_field(ui, "Enabled", ModuleName::Reader, |ui| {
                    ui.checkbox(&mut cfg.reader.enabled, "").changed()
                });
                w.labeled_config_field(ui, "Port", ModuleName::Reader, |ui| {
                    port_combobox(ui, "reader_port", &mut cfg.reader.port)
                });
            });

            let mut device_file = cfg.reader.device_file.clone().unwrap_or_default();
            w.labeled_config_field(ui, "Device file", ModuleName::Reader, |ui| {
                let changed = ui.add(egui::TextEdit::singleline(&mut device_file).desired_width(120.0)).changed();
                if changed {
                    cfg.reader.device_file = if device_file.is_empty() { None } else { Some(device_file.clone()) };
                }
                changed
            });

            let mut destinations_text = cfg.reader.destinations
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            w.labeled_config_field(ui, "Destinations", ModuleName::Reader, |ui| {
                let changed = ui.add(egui::TextEdit::singleline(&mut destinations_text).desired_width(80.0)).changed();
                if changed {
                    cfg.reader.destinations = destinations_text
                        .split(',')
                        .filter_map(|s| s.trim().parse::<u8>().ok())
                        .take(4)
                        .collect();
                }
                changed
            });
        }

    }

    fn left_body(&mut self, ui: &mut egui::Ui) {
        let mut inner = circle_container::inscribed_square(ui);
        let reader_state = self.runtime.shared_state().reader.lock().unwrap();

        inner.add_space(10.0);

        inner.vertical_centered(|ui| {
            ui.label(RichText::new("Last Card ID").strong());
            match &reader_state.last_card_id {
                Some(id) => {
                    let bytes_str = id
                        .as_bytes()
                        .chunks(2)
                        .filter_map(|chunk| {
                            std::str::from_utf8(chunk)
                                .ok()
                                .and_then(|s| u8::from_str_radix(s, 16).ok())
                        })
                        .map(|b| format!("0x{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    ui.monospace(bytes_str);
                }
                None => {
                    ui.colored_label(egui::Color32::GRAY, "None");
                }
            }
        });
    }
}
