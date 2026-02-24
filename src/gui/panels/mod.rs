use eframe::egui;

use crate::gui::components::split_panel;

pub mod config;

#[cfg(feature = "jvs")]
pub mod jvs;

#[cfg(feature = "reader")]
pub mod reader;

#[cfg(feature = "touch")]
pub mod touch;

pub trait Panel {
    fn left_header(&mut self, ui: &mut egui::Ui);
    fn left_body(&mut self, ui: &mut egui::Ui);

    fn right_header(&mut self, _ui: &mut egui::Ui) {}
    fn right_body(&mut self, _ui: &mut egui::Ui) {}
}

pub fn show_panel(ui: &mut egui::Ui, panel: &mut impl Panel, dual_screen: bool) {
    let mut layout = split_panel::split(ui, dual_screen);

    egui::ScrollArea::vertical()
        .id_salt("lh")
        .auto_shrink([false, true])
        .show(&mut layout.left_header, |ui| panel.left_header(ui));
    panel.left_body(&mut layout.left_body);

    if let Some(rh) = &mut layout.right_header {
        egui::ScrollArea::vertical()
            .id_salt("rh")
            .auto_shrink([false, true])
            .show(rh, |ui| panel.right_header(ui));
    }
    if let Some(rb) = &mut layout.right_body {
        panel.right_body(rb);
    }
}
