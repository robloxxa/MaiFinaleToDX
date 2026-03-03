use crate::config::reader::ReaderMode;
use crate::gui::components::{circle_container, module_header, port_combobox, ConfigWidgets};
use crate::gui::panels::Panel;
use crate::runtime::{ModuleName, ModuleRuntime};
use std::collections::HashSet;
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct ReaderPanelState {
    pub card_bytes: [String; 8],
}

pub struct Reader<'a> {
    runtime: &'a mut ModuleRuntime,
    pending_restarts: &'a mut HashSet<ModuleName>,
    state: &'a mut ReaderPanelState,
}

impl<'a> Reader<'a> {
    pub fn new(
        runtime: &'a mut ModuleRuntime,
        pending_restarts: &'a mut HashSet<ModuleName>,
        state: &'a mut ReaderPanelState,
    ) -> Self {
        Self { runtime, pending_restarts, state }
    }
}

impl<'a> Panel for Reader<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        let status = self.runtime.module_status(ModuleName::Reader);
        let action = {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);
            let (action, _) = module_header::show_collapsible(
                ui, ModuleName::Reader, status,
                |ui| {
                    ui.horizontal(|ui| {
                        w.labeled_config_field(ui, "Enabled", ModuleName::Reader, |ui| {
                            ui.checkbox(&mut cfg.reader.enabled, "").changed()
                        });
                        w.labeled_config_field(ui, "Mode", ModuleName::Reader, |ui| {
                            let mut changed = false;
                            egui::ComboBox::from_id_salt("reader_mode")
                                .selected_text(match cfg.reader.mode {
                                    ReaderMode::Hardware => "Hardware",
                                    ReaderMode::Emulated => "Emulated",
                                })
                                .show_ui(ui, |ui| {
                                    changed |= ui.selectable_value(
                                        &mut cfg.reader.mode,
                                        ReaderMode::Hardware,
                                        "Hardware",
                                    ).changed();
                                    changed |= ui.selectable_value(
                                        &mut cfg.reader.mode,
                                        ReaderMode::Emulated,
                                        "Emulated",
                                    ).changed();
                                });
                            changed
                        });
                    });

                    if cfg.reader.mode == ReaderMode::Hardware {
                        ui.horizontal(|ui| {
                            w.labeled_config_field(ui, "Port", ModuleName::Reader, |ui| {
                                port_combobox(ui, "reader_port", &mut cfg.reader.port)
                            });
                        });

                        let mut device_file = cfg.reader.device_file.clone().unwrap_or_default();
                        w.labeled_config_field(ui, "Device file", ModuleName::Reader, |ui| {
                            let changed = ui.add(
                                egui::TextEdit::singleline(&mut device_file).desired_width(120.0),
                            ).changed();
                            if changed {
                                cfg.reader.device_file = if device_file.is_empty() {
                                    None
                                } else {
                                    Some(device_file.clone())
                                };
                            }
                            changed
                        });

                        let mut destinations_text = cfg.reader.destinations
                            .iter()
                            .map(|b| b.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        w.labeled_config_field(ui, "Destinations", ModuleName::Reader, |ui| {
                            let changed = ui.add(
                                egui::TextEdit::singleline(&mut destinations_text).desired_width(80.0),
                            ).changed();
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
                },
            );
            action
        };
        action.apply(&mut self.runtime, ModuleName::Reader);
    }

    fn left_body(&mut self, ui: &mut egui::Ui) {
        let mut inner = circle_container::inscribed_square(ui);
        let is_emulated = self.runtime.config().reader.mode == ReaderMode::Emulated;

        inner.add_space(10.0);

        inner.vertical_centered(|ui| {
            {
                let reader_state = self.runtime.shared_state().reader.lock().unwrap();
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
            }

            if is_emulated {
                ui.add_space(12.0);
                ui.label(RichText::new("Inject Card").strong());

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    for i in 0..8usize {
                        let field_id = egui::Id::new(("reader_byte", i));
                        let s = &mut self.state.card_bytes[i];

                        let prev_len = s.len();
                        let was_empty = s.is_empty();

                        let resp = ui.add(
                            egui::TextEdit::singleline(s)
                                .id(field_id)
                                .desired_width(22.0)
                                .hint_text("00"),
                        );

                        // Remove any invalid chars the user just typed.
                        let raw_len = s.len();
                        s.retain(|c| c.is_ascii_hexdigit());
                        s.make_ascii_uppercase();
                        if s.len() > 2 { s.truncate(2); }

                        // Invalid chars were typed — reset cursor to end.
                        if s.len() < raw_len {
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), field_id) {
                                let end = egui::text::CCursor::new(s.len());
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(end)));
                                state.store(ui.ctx(), field_id);
                            }
                        }

                        // Auto-advance to next field when two chars are entered.
                        if resp.changed() && s.len() == 2 && s.len() > prev_len && i < 7 {
                            ui.ctx().memory_mut(|m| {
                                m.request_focus(egui::Id::new(("reader_byte", i + 1)));
                            });
                        }

                        // Backspace on empty field → go to previous field.
                        if was_empty && i > 0 && resp.has_focus() {
                            if ui.ctx().input(|inp| inp.key_pressed(egui::Key::Backspace)) {
                                ui.ctx().memory_mut(|m| {
                                    m.request_focus(egui::Id::new(("reader_byte", i - 1)));
                                });
                            }
                        }
                    }
                });

                ui.add_space(4.0);
                let all_filled = self.state.card_bytes.iter().all(|s| s.len() == 2);
                ui.add_enabled_ui(all_filled, |ui| {
                    if ui.button("Scan").clicked() {
                        let bytes: Option<[u8; 8]> = self.state.card_bytes
                            .iter()
                            .map(|s| u8::from_str_radix(s, 16).ok())
                            .collect::<Option<Vec<_>>>()
                            .and_then(|v| v.try_into().ok());
                        if let Some(card) = bytes {
                            self.runtime.shared_state().reader.lock().unwrap().pending_card = Some(card);
                        }
                    }
                });
            }
        });
    }
}
