use crate::{gui::components::module_header, runtime::{ModuleName, ModuleRuntime}, state::SharedState};
use eframe::egui;
use std::f32::consts::TAU;

pub fn show(ui: &mut egui::Ui, runtime: &mut ModuleRuntime) {
    module_header::show(ui, runtime, ModuleName::Touch);
    
    ui.separator();

    let state = runtime.shared_state();
    
    let p1_finale = state.touch.load_p1_finale();
    let p2_finale = state.touch.load_p2_finale();
    let p1_dx = state.touch.load_p1_dx();
    let p2_dx = state.touch.load_p2_dx();

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.strong("Player 1");
            draw_touch_circle(ui, "p1_circle", &p1_finale);
            ui.label(format!("FiNALE: {:02X?}", p1_finale));
            ui.label(format!("DX:     {:02X?}", &p1_dx[1..8]));
        });

        ui.add_space(20.0);

        ui.vertical(|ui| {
            ui.strong("Player 2");
            draw_touch_circle(ui, "p2_circle", &p2_finale);
            ui.label(format!("FiNALE: {:02X?}", p2_finale));
            ui.label(format!("DX:     {:02X?}", &p2_dx[1..8]));
        });
    });
}

fn draw_touch_circle(ui: &mut egui::Ui, _id: &str, raw: &[u8; 4]) {
    let desired_size = egui::vec2(200.0, 200.0);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let center = response.rect.center();
    let radius = 90.0;

    let inactive_color = egui::Color32::from_gray(60);
    let active_color = egui::Color32::from_rgb(0, 200, 100);

    // A zones (outer ring) - 8 sectors
    let a_inner = radius * 0.65;
    let a_outer = radius;
    for i in 0..8 {
        let active = is_zone_active(raw, i, true);
        let color = if active { active_color } else { inactive_color };
        draw_sector(&painter, center, a_inner, a_outer, i, 8, color);
    }

    // B zones (inner ring) - 8 sectors
    let b_inner = radius * 0.3;
    let b_outer = radius * 0.6;
    for i in 0..8 {
        let active = is_zone_active(raw, i, false);
        let color = if active { active_color } else { inactive_color };
        draw_sector(&painter, center, b_inner, b_outer, i, 8, color);
    }

    // C zone (center)
    let c_active = raw[3] & 0x10 != 0;
    let c_color = if c_active { active_color } else { inactive_color };
    painter.circle_filled(center, radius * 0.25, c_color);

    // Zone labels
    let label_r = radius * 0.82;
    for i in 0..8 {
        let angle = sector_mid_angle(i, 8);
        let pos = center + egui::vec2(angle.cos() * label_r, angle.sin() * label_r);
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            format!("A{}", i + 1),
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
    }

    let label_r = radius * 0.47;
    for i in 0..8 {
        let angle = sector_mid_angle(i, 8);
        let pos = center + egui::vec2(angle.cos() * label_r, angle.sin() * label_r);
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            format!("B{}", i + 1),
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
    }

    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "C",
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );
}

fn is_zone_active(raw: &[u8; 4], zone: usize, is_a: bool) -> bool {
    // FiNALE touch encoding: each byte contains bits for different zones
    // A zones use even letters (A,C,E,G,I,K,M,O), B zones use odd (B,D,F,H,J,L,N,P)
    // The mapping depends on the protocol, approximate for visualization:
    let byte_idx = zone / 2;
    let bit_idx = if is_a { (zone % 2) * 2 } else { (zone % 2) * 2 + 1 };
    if byte_idx < 4 {
        raw[byte_idx] & (1 << bit_idx) != 0
    } else {
        false
    }
}

fn sector_mid_angle(idx: usize, total: usize) -> f32 {
    let start = -TAU / 4.0 + (idx as f32 / total as f32) * TAU;
    start + TAU / (2.0 * total as f32)
}

fn draw_sector(
    painter: &egui::Painter,
    center: egui::Pos2,
    inner_r: f32,
    outer_r: f32,
    idx: usize,
    total: usize,
    color: egui::Color32,
) {
    let gap = 0.02;
    let start_angle = -TAU / 4.0 + (idx as f32 / total as f32) * TAU + gap;
    let end_angle = -TAU / 4.0 + ((idx + 1) as f32 / total as f32) * TAU - gap;

    let steps = 16;
    let mut points = Vec::with_capacity(steps * 2 + 2);

    // Outer arc
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let angle = start_angle + (end_angle - start_angle) * t;
        points.push(center + egui::vec2(angle.cos() * outer_r, angle.sin() * outer_r));
    }

    // Inner arc (reverse)
    for s in (0..=steps).rev() {
        let t = s as f32 / steps as f32;
        let angle = start_angle + (end_angle - start_angle) * t;
        points.push(center + egui::vec2(angle.cos() * inner_r, angle.sin() * inner_r));
    }

    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
    ));
}
