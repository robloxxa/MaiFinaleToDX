mod editor;
mod touch_circle;
mod zone_shape;

pub use editor::TouchEditorState;
use editor::show_zone_editor;
use touch_circle::draw_touch_circle;

use crate::config::touch::TouchMode;
use crate::gui::components::{module_header, port_combobox, ConfigWidgets};
use crate::gui::panels::Panel;
use crate::runtime::{Module, ModuleName, ModuleRuntime};
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
        Self { runtime, editor, pending_restarts }
    }
}

const DELUXE_MODULES: [ModuleName; 2] = [
    ModuleName::TouchDeluxe(1),
    ModuleName::TouchDeluxe(2),
];

fn module_row(
    ui: &mut egui::Ui,
    label: &str,
    enabled: &mut bool,
    mode: &mut TouchMode,
    port: &mut String,
    w: &mut ConfigWidgets<'_>,
) {
    ui.horizontal(|ui| {
        w.labeled_config_field(ui, label, ModuleName::TouchFinale, |ui| {
            ui.checkbox(enabled, "").changed()
        });
        w.labeled_config_field(ui, "Mode", ModuleName::TouchFinale, |ui| {
            let mut changed = false;
            egui::ComboBox::from_id_salt(label)
                .selected_text(match mode {
                    TouchMode::Hardware => "Hardware",
                    TouchMode::Emulated => "Emulated",
                })
                .show_ui(ui, |ui| {
                    changed |= ui.selectable_value(mode, TouchMode::Hardware, "Hardware").changed();
                    changed |= ui.selectable_value(mode, TouchMode::Emulated, "Emulated").changed();
                });
            changed
        });
        if *mode == TouchMode::Hardware {
            w.labeled_config_field(ui, "Port", ModuleName::TouchFinale, |ui| {
                port_combobox(ui, (label, "port"), port)
            });
        }
    });
}

fn dx_module_row(
    ui: &mut egui::Ui,
    enabled: &mut bool,
    mode: &mut TouchMode,
    p1_port: &mut String,
    p2_port: &mut String,
    w: &mut ConfigWidgets<'_>,
) {
    ui.horizontal(|ui| {
        w.labeled_config_field(ui, "DX", DELUXE_MODULES.to_vec(), |ui| {
            ui.checkbox(enabled, "").changed()
        });
        w.labeled_config_field(ui, "Mode", DELUXE_MODULES.to_vec(), |ui| {
            let mut changed = false;
            egui::ComboBox::from_id_salt("dx_mode")
                .selected_text(match mode {
                    TouchMode::Hardware => "Hardware",
                    TouchMode::Emulated => "Emulated",
                })
                .show_ui(ui, |ui| {
                    changed |= ui.selectable_value(mode, TouchMode::Hardware, "Hardware").changed();
                    changed |= ui.selectable_value(mode, TouchMode::Emulated, "Emulated").changed();
                });
            changed
        });
        if *mode == TouchMode::Hardware {
            w.labeled_config_field(ui, "P1 Port", ModuleName::TouchDeluxe(1), |ui| {
                port_combobox(ui, "dx_p1_port", p1_port)
            });
            w.labeled_config_field(ui, "P2 Port", ModuleName::TouchDeluxe(2), |ui| {
                port_combobox(ui, "dx_p2_port", p2_port)
            });
        }
    });
}

impl<'a> Panel for Touch<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        module_header::show(ui, self.runtime, ModuleName::TouchFinale);
        module_header::show(ui, self.runtime, ModuleName::TouchDeluxe(1));

        let cfg = self.runtime.config_mut();
        let mut w = ConfigWidgets::new(self.pending_restarts);
        module_row(
            ui,
            "Finale",
            &mut cfg.touch.finale.enabled,
            &mut cfg.touch.finale.mode,
            &mut cfg.touch.finale.port,
            &mut w,
        );
        dx_module_row(
            ui,
            &mut cfg.touch.dx.enabled,
            &mut cfg.touch.dx.mode,
            &mut cfg.touch.dx.p1_port,
            &mut cfg.touch.dx.p2_port,
            &mut w,
        );
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
        module_header::show(ui, self.runtime, ModuleName::TouchDeluxe(2));
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
