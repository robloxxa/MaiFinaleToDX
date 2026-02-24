use eframe::egui::{self, Ui, UiBuilder};

/// Returns a child `Ui` for the largest square inscribed in a circle
/// that fits the given square body `Ui`. Side = diameter / sqrt(2).
pub fn inscribed_square(ui: &mut Ui) -> Ui {
    let rect = ui.available_rect_before_wrap();
    let diameter = rect.width().min(rect.height());
    let side = diameter / std::f32::consts::SQRT_2;

    let center = rect.center();
    let inner = egui::Rect::from_center_size(center, egui::vec2(side, side));

    ui.allocate_rect(rect, egui::Sense::hover());
    ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
    )
}

/// Returns a child `Ui` for the widest rectangle inscribed in a circle
/// that fits the given square body `Ui`. Width = diameter, height = diameter / sqrt(2).
pub fn inscribed_rect(ui: &mut Ui) -> Ui {
    let rect = ui.available_rect_before_wrap();
    let diameter = rect.width().min(rect.height());
    let width = diameter;
    let height = diameter / std::f32::consts::SQRT_2;

    let center = rect.center();
    let inner = egui::Rect::from_center_size(center, egui::vec2(width, height));

    ui.allocate_rect(rect, egui::Sense::hover());
    ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
    )
}
