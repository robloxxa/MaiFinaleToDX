use std::f32::consts::PI;

use winapi::ctypes::c_int;

use crate::gui::components::{module_header, port_combobox, ConfigWidgets};
use crate::gui::panels::Panel;
use crate::runtime::{ModuleName, ModuleRuntime};
use std::collections::HashSet;
use eframe::egui;
use eframe::epaint::{Mesh, PathShape, PathStroke};

pub struct Jvs<'a> {
    runtime: &'a mut ModuleRuntime,
    pending_restarts: &'a mut HashSet<ModuleName>,
}

impl<'a> Jvs<'a> {
    pub fn new(
        runtime: &'a mut ModuleRuntime,
        pending_restarts: &'a mut HashSet<ModuleName>,
    ) -> Self {
        Self { runtime, pending_restarts }
    }
}

impl<'a> Panel for Jvs<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        let status = self.runtime.module_status(ModuleName::Jvs);
        let action = {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);
            let (action, _) = module_header::show_collapsible(
                ui, ModuleName::Jvs, status, Some(&mut cfg.jvs.enabled),
                |ui| {
                    ui.horizontal(|ui| {
                        w.labeled_config_field(ui, "Port", ModuleName::Jvs, |ui| {
                            port_combobox(ui, "jvs_port", &mut cfg.jvs.port)
                        });
                    });
                    egui::CollapsingHeader::new("Key Bindings").show(ui, |ui| {
                        egui::Grid::new("jvs_keys").num_columns(2).show(ui, |ui| {
                            key_bind_row(ui, "Test", &mut cfg.jvs.input.test, &mut w);
                            key_bind_row(ui, "Service", &mut cfg.jvs.input.service, &mut w);
                            for i in 1..=8u8 {
                                key_bind_row(ui, &format!("P1 Btn {i}"), p1_btn_mut(&mut cfg.jvs.input, i), &mut w);
                            }
                            for i in 1..=8u8 {
                                key_bind_row(ui, &format!("P2 Btn {i}"), p2_btn_mut(&mut cfg.jvs.input, i), &mut w);
                            }
                        });
                    });
                },
            );
            action
        };
        action.apply(&mut self.runtime, ModuleName::Jvs);
    }

    fn left_body(&mut self, ui: &mut egui::Ui) {
        let buttons = self.runtime.shared_state().jvs.load_buttons();
        draw_jvs_circle(ui, &buttons.p1, buttons.test, buttons.service, "P1");
    }

    fn right_body(&mut self, ui: &mut egui::Ui) {
        let buttons = self.runtime.shared_state().jvs.load_buttons();
        draw_jvs_circle(ui, &buttons.p2, buttons.test, buttons.service, "P2");
    }
}

fn draw_jvs_circle(ui: &mut egui::Ui, btns: &[bool; 8], test: bool, service: bool, player: &str) {
    let available = ui.available_size();
    let size = available.x.min(available.y);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = (size - 2.0) / 2.0;

    // 8 button zones, same angular layout as A zones in touch panel.
    // Radial depth is 2.5x shorter than original A zones (0.40r → 0.16r).
    for i in 0..8usize {
        let angle = -PI / 2.0 + i as f32 * PI / 4.0 + PI / 8.0;
        draw_btn_zone(&painter, center, radius, angle, btns[i], &format!("Btn {}", i + 1));
    }

    draw_center_text(&painter, center, radius, test, service, player);
}

fn draw_btn_zone(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    angle: f32,
    pressed: bool,
    label: &str,
) {
    // Same shape as A zones: wide outer arc tapering to a narrow inner edge.
    // Radial depth is 2.5x shorter (0.40r → 0.16r), so half_inner is compressed
    // to 4° (vs 7° in A zones) to keep the trapezoidal taper clearly visible.
    let half_outer = 14.0_f32.to_radians();
    let half_inner =7.0_f32.to_radians();
    let outer_r = radius;
    let mid_r = radius * 0.86;
    let inner_r = radius * 0.82;

    const ARC_STEPS: usize = 8;
    let arc: Vec<egui::Pos2> = (0..=ARC_STEPS)
        .map(|i| {
            polar(
                center,
                (angle - half_outer) + i as f32 / ARC_STEPS as f32 * 2.0 * half_outer,
                outer_r,
            )
        })
        .collect();

    let tr = polar(center, angle + half_outer, mid_r);
    let br = polar(center, angle + half_inner, inner_r);
    let bl = polar(center, angle - half_inner, inner_r);
    let tl = polar(center, angle - half_outer, mid_r);

    let points: Vec<egui::Pos2> = arc.into_iter().chain([tr, br, bl, tl]).collect();

    let fill = if pressed {
        egui::Color32::from_rgba_unmultiplied(60, 220, 60, 200)
    } else {
        egui::Color32::from_rgba_unmultiplied(55, 55, 55, 180)
    };
    let stroke_color = if pressed {
        egui::Color32::from_rgba_unmultiplied(80, 230, 80, 255)
    } else {
        egui::Color32::from_gray(130)
    };

    let mut mesh = Mesh::default();
    for &pos in &points {
        mesh.colored_vertex(pos, fill);
    }
    let n = mesh.vertices.len() as u32;
    for i in 1..n - 1 {
        mesh.add_triangle(0, i, i + 1);
    }
    painter.add(egui::Shape::Mesh(mesh.into()));

    painter.add(egui::Shape::Path(PathShape {
        points: points.clone(),
        closed: true,
        fill: egui::Color32::TRANSPARENT,
        stroke: PathStroke::new(1.5, stroke_color),
    }));

    let label_pos = polar(center, angle, radius * 0.92);
    let font_size = (radius * 0.075).clamp(7.0, 11.0);
    painter.text(
        label_pos,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        egui::Color32::WHITE,
    );
}

fn draw_center_text(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    test: bool,
    service: bool,
    player: &str,
) {
    let line_h = (radius * 0.10).clamp(8.0, 14.0);
    let label_size = (radius * 0.08).clamp(7.0, 11.0);

    let active_color = egui::Color32::from_rgb(60, 220, 60);
    let inactive_color = egui::Color32::from_gray(90);

    painter.text(
        egui::pos2(center.x, center.y - line_h),
        egui::Align2::CENTER_CENTER,
        "Test",
        egui::FontId::proportional(line_h),
        if test { active_color } else { inactive_color },
    );
    painter.text(
        egui::pos2(center.x, center.y + line_h * 0.1),
        egui::Align2::CENTER_CENTER,
        "Service",
        egui::FontId::proportional(line_h),
        if service { active_color } else { inactive_color },
    );
    painter.text(
        egui::pos2(center.x, center.y + line_h * 1.4),
        egui::Align2::CENTER_CENTER,
        player,
        egui::FontId::proportional(label_size),
        egui::Color32::from_gray(130),
    );
}

fn polar(center: egui::Pos2, angle: f32, r: f32) -> egui::Pos2 {
    center + egui::vec2(angle.cos() * r, angle.sin() * r)
}

fn key_bind_row(ui: &mut egui::Ui, label: &str, vk: &mut c_int, w: &mut ConfigWidgets<'_>) {
    ui.label(label);
    let mut text = format!("0x{:02X}", vk);
    if ui.add(egui::TextEdit::singleline(&mut text).desired_width(55.0)).changed() {
        if let Ok(v) = i32::from_str_radix(text.trim_start_matches("0x"), 16) {
            *vk = v;
            w.mark_dirty(ModuleName::Jvs);
        }
    }
    ui.end_row();
}

fn p1_btn_mut(input: &mut crate::config::jvs::Input, n: u8) -> &mut c_int {
    match n {
        1 => &mut input.p1_btn1,
        2 => &mut input.p1_btn2,
        3 => &mut input.p1_btn3,
        4 => &mut input.p1_btn4,
        5 => &mut input.p1_btn5,
        6 => &mut input.p1_btn6,
        7 => &mut input.p1_btn7,
        _ => &mut input.p1_btn8,
    }
}

fn p2_btn_mut(input: &mut crate::config::jvs::Input, n: u8) -> &mut c_int {
    match n {
        1 => &mut input.p2_btn1,
        2 => &mut input.p2_btn2,
        3 => &mut input.p2_btn3,
        4 => &mut input.p2_btn4,
        5 => &mut input.p2_btn5,
        6 => &mut input.p2_btn6,
        7 => &mut input.p2_btn7,
        _ => &mut input.p2_btn8,
    }
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
