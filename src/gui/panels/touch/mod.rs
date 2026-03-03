mod editor;
mod touch_circle;
mod zone_shape;

use editor::show_zone_editor;
pub use editor::TouchEditorState;
use touch_circle::draw_touch_circle;

use crate::config::touch::TouchMode;
use crate::gui::components::{module_header, port_combobox, port_combobox_optional, ConfigWidgets};
use crate::gui::panels::Panel;
use crate::runtime::{ModuleName, ModuleRuntime};
use eframe::egui;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    P1,
    P2,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::P1 => write!(f, "Player 1"),
            Player::P2 => write!(f, "Player 2"),
        }
    }
}

pub struct Touch<'a> {
    runtime: &'a mut ModuleRuntime,
    editor: &'a mut TouchEditorState,
    pending_restarts: &'a mut HashSet<ModuleName>,
}

impl<'a> Touch<'a> {
    pub fn new(
        runtime: &'a mut ModuleRuntime,
        editor: &'a mut TouchEditorState,
        pending_restarts: &'a mut HashSet<ModuleName>,
    ) -> Self {
        Self {
            runtime,
            editor,
            pending_restarts,
        }
    }
}

const DELUXE_MODULES: [ModuleName; 2] = [ModuleName::TouchDeluxe(1), ModuleName::TouchDeluxe(2)];

impl<'a> Panel for Touch<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        let finale_status = self.runtime.module_status(ModuleName::TouchFinale);
        let finale_action = {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);
            let (action, _) =
                module_header::show_collapsible(ui, ModuleName::TouchFinale, finale_status, |ui| {
                    ui.horizontal(|ui| {
                        w.labeled_config_field(ui, "Enabled", ModuleName::TouchFinale, |ui| {
                            ui.checkbox(&mut cfg.touch.finale.enabled, "").changed()
                        });
                        w.labeled_config_field(ui, "Mode", ModuleName::TouchFinale, |ui| {
                            let mut changed = false;
                            egui::ComboBox::from_id_salt("finale_mode")
                                .selected_text(match cfg.touch.finale.mode {
                                    TouchMode::Hardware => "Hardware",
                                    TouchMode::Emulated => "Emulated",
                                })
                                .show_ui(ui, |ui| {
                                    changed |= ui
                                        .selectable_value(
                                            &mut cfg.touch.finale.mode,
                                            TouchMode::Hardware,
                                            "Hardware",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut cfg.touch.finale.mode,
                                            TouchMode::Emulated,
                                            "Emulated",
                                        )
                                        .changed();
                                });
                            changed
                        });
                        if cfg.touch.finale.mode == TouchMode::Hardware {
                            w.labeled_config_field(ui, "Port", ModuleName::TouchFinale, |ui| {
                                port_combobox(ui, "finale_port", &mut cfg.touch.finale.port)
                            });
                        }
                    });
                });
            action
        };
        finale_action.apply(&mut self.runtime, ModuleName::TouchFinale);

        let dx_p1_status = self.runtime.module_status(ModuleName::TouchDeluxe(1));
        let dx_p1_action = {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);
            let (action, _) = module_header::show_collapsible(
                ui,
                ModuleName::TouchDeluxe(1),
                dx_p1_status,
                |ui| {
                    ui.horizontal(|ui| {
                        w.labeled_config_field(ui, "Enabled", DELUXE_MODULES.to_vec(), |ui| {
                            ui.checkbox(&mut cfg.touch.dx.enabled, "").changed()
                        });
                        w.labeled_config_field(ui, "Mode", DELUXE_MODULES.to_vec(), |ui| {
                            let mut changed = false;
                            egui::ComboBox::from_id_salt("dx_mode")
                                .selected_text(match cfg.touch.dx.mode {
                                    TouchMode::Hardware => "Hardware",
                                    TouchMode::Emulated => "Emulated",
                                })
                                .show_ui(ui, |ui| {
                                    changed |= ui
                                        .selectable_value(
                                            &mut cfg.touch.dx.mode,
                                            TouchMode::Hardware,
                                            "Hardware",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut cfg.touch.dx.mode,
                                            TouchMode::Emulated,
                                            "Emulated",
                                        )
                                        .changed();
                                });
                            changed
                        });
                        if cfg.touch.dx.mode == TouchMode::Hardware {
                            w.labeled_config_field(
                                ui,
                                "P1 Port",
                                ModuleName::TouchDeluxe(1),
                                |ui| {
                                    port_combobox_optional(
                                        ui,
                                        "dx_p1_port",
                                        &mut cfg.touch.dx.p1_port,
                                    )
                                },
                            );
                        };
                    });
                },
            );
            action
        };
        dx_p1_action.apply(&mut self.runtime, ModuleName::TouchDeluxe(1));
    }

    fn left_body(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y);
        let touch = &self.runtime.shared_state().touch;
        let finale_hw = touch.load_p1_finale_hw();
        let dx = touch.load_p1_dx();
        let gui = draw_touch_circle(ui, &finale_hw, &dx, size, Player::P1, self.editor);
        self.runtime.shared_state().touch.store_p1_finale_gui(gui);

        let ctx = ui.ctx().clone();
        show_zone_editor(&ctx, self.runtime, self.editor, self.pending_restarts);
    }

    fn right_header(&mut self, ui: &mut egui::Ui) {
        let status = self.runtime.module_status(ModuleName::TouchDeluxe(2));
        let action = {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);
            let (action, _) =
                module_header::show_collapsible(ui, ModuleName::TouchDeluxe(2), status, |ui| {
                    ui.horizontal(|ui| {
                        if cfg.touch.dx.mode == TouchMode::Hardware {
                            w.labeled_config_field(
                                ui,
                                "P2 Port",
                                ModuleName::TouchDeluxe(2),
                                |ui| {
                                    port_combobox_optional(
                                        ui,
                                        "dx_p2_port",
                                        &mut cfg.touch.dx.p2_port,
                                    )
                                },
                            );
                        };
                    });
                });
            action
        };
        action.apply(&mut self.runtime, ModuleName::TouchDeluxe(2));
    }

    fn right_body(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y);
        let touch = &self.runtime.shared_state().touch;
        let finale_hw = touch.load_p2_finale_hw();
        let dx = touch.load_p2_dx();
        let gui = draw_touch_circle(ui, &finale_hw, &dx, size, Player::P2, self.editor);
        self.runtime.shared_state().touch.store_p2_finale_gui(gui);
    }
}
