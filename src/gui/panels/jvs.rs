use crate::{gui::components::module_header, runtime::{ModuleName, ModuleRuntime}, state::SharedState};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, runtime: &mut ModuleRuntime) {
    module_header::show(ui, runtime, ModuleName::Jvs);
    
    ui.separator();

    let buttons = runtime.shared_state().jvs.load_buttons();

    egui::Grid::new("jvs_buttons_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            button_indicator(ui, "Test", buttons.test);
            button_indicator(ui, "Service", buttons.service);
            ui.end_row();
        });

    ui.add_space(10.0);

    ui.columns(2, |cols| {
        cols[0].strong("Player 1");
        egui::Grid::new("p1_buttons")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(&mut cols[0], |ui| {
                for i in 0..8 {
                    button_indicator(ui, &format!("Btn {}", i + 1), buttons.p1[i]);
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });

        cols[1].strong("Player 2");
        egui::Grid::new("p2_buttons")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(&mut cols[1], |ui| {
                for i in 0..8 {
                    button_indicator(ui, &format!("Btn {}", i + 1), buttons.p2[i]);
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    });
}

fn button_indicator(ui: &mut egui::Ui, label: &str, pressed: bool) {
    let color = if pressed {
        egui::Color32::GREEN
    } else {
        egui::Color32::from_gray(60)
    };

    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 5.0, color);
        ui.label(label);
    });
}
