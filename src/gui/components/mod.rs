pub mod circle_container;
pub mod module_header;
pub mod port_selector;
pub mod split_panel;

pub use port_selector::{port_combobox, port_combobox_optional};

use eframe::egui;
use std::collections::HashSet;

use crate::runtime::ModuleName;

pub trait IntoModuleList {
    fn into_module_list(self) -> Vec<ModuleName>;
}

impl IntoModuleList for ModuleName {
    fn into_module_list(self) -> Vec<ModuleName> {
        vec![self]
    }
}

impl IntoModuleList for Vec<ModuleName> {
    fn into_module_list(self) -> Vec<ModuleName> {
        self
    }
}

impl IntoModuleList for Option<ModuleName> {
    fn into_module_list(self) -> Vec<ModuleName> {
        self.into_iter().collect()
    }
}

pub struct ConfigWidgets<'a> {
    pending: &'a mut HashSet<ModuleName>,
}

impl<'a> ConfigWidgets<'a> {
    pub fn new(pending: &'a mut HashSet<ModuleName>) -> Self {
        Self { pending }
    }

    pub fn mark_dirty(&mut self, module: ModuleName) {
        self.pending.insert(module);
    }

    pub fn labeled_config_field(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        modules: impl IntoModuleList,
        add_widget: impl FnOnce(&mut egui::Ui) -> bool,
    ) {
        let pending = &mut *self.pending;
        let modules = modules.into_module_list();
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).color(egui::Color32::from_gray(160)));
            if add_widget(ui) {
                for module in modules {
                    pending.insert(module);
                }
            }
        });
    }
}
