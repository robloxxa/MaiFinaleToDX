use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use winapi::ctypes::c_int;

use crate::config::jvs::JvsMode;
use crate::gui::components::{module_header, port_combobox, ConfigWidgets};
use crate::gui::panels::Panel;
use crate::jvs::State as JvsState;
use crate::runtime::{ModuleName, ModuleRuntime};
use eframe::egui;
use eframe::epaint::{Mesh, PathShape, PathStroke};
use std::collections::HashSet;

pub struct Jvs<'a> {
    runtime: &'a mut ModuleRuntime,
    pending_restarts: &'a mut HashSet<ModuleName>,
}

impl<'a> Jvs<'a> {
    pub fn new(
        runtime: &'a mut ModuleRuntime,
        pending_restarts: &'a mut HashSet<ModuleName>,
    ) -> Self {
        Self {
            runtime,
            pending_restarts,
        }
    }
}

impl<'a> Panel for Jvs<'a> {
    fn left_header(&mut self, ui: &mut egui::Ui) {
        let status = self.runtime.module_status(ModuleName::Jvs);
        let action = {
            let cfg = self.runtime.config_mut();
            let mut w = ConfigWidgets::new(self.pending_restarts);
            let (action, _) = module_header::show_collapsible(ui, ModuleName::Jvs, status, |ui| {
                ui.horizontal(|ui| {
                    w.labeled_config_field(ui, "Enabled", ModuleName::Jvs, |ui| {
                        ui.checkbox(&mut cfg.jvs.enabled, "").changed()
                    });
                    w.labeled_config_field(ui, "Mode", ModuleName::Jvs, |ui| {
                        let mut changed = false;
                        egui::ComboBox::from_id_salt("jvs_mode")
                            .selected_text(match cfg.jvs.mode {
                                JvsMode::Hardware => "Hardware",
                                JvsMode::Emulated => "Emulated",
                            })
                            .show_ui(ui, |ui| {
                                changed |= ui
                                    .selectable_value(
                                        &mut cfg.jvs.mode,
                                        JvsMode::Hardware,
                                        "Hardware",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut cfg.jvs.mode,
                                        JvsMode::Emulated,
                                        "Emulated",
                                    )
                                    .changed();
                            });
                        changed
                    });
                });

                if cfg.jvs.mode == JvsMode::Hardware {
                    ui.horizontal(|ui| {
                        w.labeled_config_field(ui, "Port", ModuleName::Jvs, |ui| {
                            port_combobox(ui, "jvs_port", &mut cfg.jvs.port)
                        });
                    });
                }

                egui::CollapsingHeader::new("Key Bindings").show(ui, |ui| {
                    egui::Grid::new("jvs_keys").num_columns(2).show(ui, |ui| {
                        key_bind_row(ui, "Test", &mut cfg.jvs.input.test, &mut w);
                        key_bind_row(ui, "Service", &mut cfg.jvs.input.service, &mut w);
                        for i in 1..=8u8 {
                            key_bind_row(
                                ui,
                                &format!("P1 Btn {i}"),
                                p1_btn_mut(&mut cfg.jvs.input, i),
                                &mut w,
                            );
                        }
                        for i in 1..=8u8 {
                            key_bind_row(
                                ui,
                                &format!("P2 Btn {i}"),
                                p2_btn_mut(&mut cfg.jvs.input, i),
                                &mut w,
                            );
                        }
                    });
                });
            });
            action
        };
        action.apply(&mut self.runtime, ModuleName::Jvs);
    }

    fn left_body(&mut self, ui: &mut egui::Ui) {
        let jvs_state = self.runtime.shared_state().jvs.clone();
        jvs_state.gui_active.store(true, Ordering::Relaxed);
        let buttons = jvs_state.load_buttons();
        let input = self.runtime.config().jvs.input.clone();
        let p1_vks = [
            input.p1_btn1,
            input.p1_btn2,
            input.p1_btn3,
            input.p1_btn4,
            input.p1_btn5,
            input.p1_btn6,
            input.p1_btn7,
            input.p1_btn8,
        ];
        let all_vks = [
            input.test,
            input.service,
            input.p1_btn1,
            input.p1_btn2,
            input.p1_btn3,
            input.p1_btn4,
            input.p1_btn5,
            input.p1_btn6,
            input.p1_btn7,
            input.p1_btn8,
        ];
        draw_jvs_circle(
            ui,
            &buttons.p1,
            buttons.test,
            buttons.service,
            "P1",
            &jvs_state,
            true,
            &p1_vks,
            &all_vks,
        );
    }

    fn right_body(&mut self, ui: &mut egui::Ui) {
        let jvs_state = self.runtime.shared_state().jvs.clone();
        let buttons = jvs_state.load_buttons();
        let input = self.runtime.config().jvs.input.clone();
        let p2_vks = [
            input.p2_btn1,
            input.p2_btn2,
            input.p2_btn3,
            input.p2_btn4,
            input.p2_btn5,
            input.p2_btn6,
            input.p2_btn7,
            input.p2_btn8,
        ];
        let all_vks = [
            input.test,
            input.service,
            input.p2_btn1,
            input.p2_btn2,
            input.p2_btn3,
            input.p2_btn4,
            input.p2_btn5,
            input.p2_btn6,
            input.p2_btn7,
            input.p2_btn8,
        ];
        draw_jvs_circle(
            ui,
            &buttons.p2,
            buttons.test,
            buttons.service,
            "P2",
            &jvs_state,
            false,
            &p2_vks,
            &all_vks,
        );
    }
}

fn draw_jvs_circle(
    ui: &mut egui::Ui,
    btns: &[bool; 8],
    test: bool,
    service: bool,
    player: &str,
    jvs_state: &Arc<JvsState>,
    is_p1: bool,
    vk_keys: &[c_int; 8],
    all_vks: &[c_int],
) {
    let available = ui.available_size();
    let size = available.x.min(available.y);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

    let center = rect.center();
    let radius = (size - 2.0) / 2.0;

    // 8 button zones, same angular layout as A zones in touch panel.
    // Radial depth is 2.5x shorter than original A zones (0.40r → 0.16r).
    for i in 0..8usize {
        let angle = -PI / 2.0 + i as f32 * PI / 4.0 + PI / 8.0;
        // bit position: P1 buttons start at bit 2, P2 at bit 10
        let bit = if is_p1 { 2 + i as u8 } else { 10 + i as u8 };
        draw_btn_zone(
            ui,
            rect,
            center,
            radius,
            angle,
            btns[i],
            i + 1,
            vk_keys[i],
            jvs_state,
            bit,
        );
    }

    // Collect currently pressed keys from the mapping via GetAsyncKeyState
    let pressed: Vec<String> = all_vks
        .iter()
        .filter(|&&vk| key_is_down(vk))
        .map(|&vk| vk_label(vk))
        .collect();

    let painter = ui.painter_at(rect);
    draw_center_text(&painter, center, radius, test, service, player, &pressed);
}

fn draw_btn_zone(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    center: egui::Pos2,
    radius: f32,
    angle: f32,
    pressed: bool,
    btn_num: usize,
    vk: c_int,
    jvs_state: &Arc<JvsState>,
    bit: u8,
) {
    let half_outer = 14.0_f32.to_radians();
    let half_inner = 7.0_f32.to_radians();
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

    // Compute a bounding box for the zone polygon to allocate a response area
    {
        let (min_x, max_x, min_y, max_y) = points.iter().fold(
            (
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
            ),
            |(x0, x1, y0, y1), p| (x0.min(p.x), x1.max(p.x), y0.min(p.y), y1.max(p.y)),
        );
        let zone_rect =
            egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));
        // Clamp to the outer rect to prevent allocating outside widget area
        let zone_rect = zone_rect.intersect(rect);
        let response = ui.allocate_rect(zone_rect, egui::Sense::hover());
        let held = response.contains_pointer() && ui.ctx().input(|i| i.pointer.primary_down());
        jvs_state.set_button(bit, held);
    }

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

    let painter = ui.painter_at(rect);

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

    let font_size = (radius * 0.075).clamp(7.0, 11.0);
    // VK label (e.g. "W", "N8") at outer position
    painter.text(
        polar(center, angle, radius * 0.93),
        egui::Align2::CENTER_CENTER,
        vk_label(vk),
        egui::FontId::proportional(font_size),
        egui::Color32::WHITE,
    );
    // Button number at inner position
    painter.text(
        polar(center, angle, radius * 0.84),
        egui::Align2::CENTER_CENTER,
        format!("{}", btn_num),
        egui::FontId::proportional(font_size * 0.85),
        egui::Color32::from_gray(160),
    );
}

fn draw_center_text(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    test: bool,
    service: bool,
    player: &str,
    pressed_keys: &[String],
) {
    let line_h = (radius * 0.10).clamp(8.0, 14.0);
    let label_size = (radius * 0.08).clamp(7.0, 11.0);

    let active_color = egui::Color32::from_rgb(60, 220, 60);
    let inactive_color = egui::Color32::from_gray(90);

    // Test / Service state
    painter.text(
        egui::pos2(center.x, center.y - line_h * 1.2),
        egui::Align2::CENTER_CENTER,
        "Test",
        egui::FontId::proportional(line_h),
        if test { active_color } else { inactive_color },
    );
    painter.text(
        egui::pos2(center.x, center.y - line_h * 0.1),
        egui::Align2::CENTER_CENTER,
        "Service",
        egui::FontId::proportional(line_h),
        if service {
            active_color
        } else {
            inactive_color
        },
    );

    // Currently pressed keyboard keys from the mapping
    let keys_str = if pressed_keys.is_empty() {
        String::new()
    } else {
        pressed_keys.join("  ")
    };
    let keys_color = if pressed_keys.is_empty() {
        egui::Color32::from_gray(55)
    } else {
        egui::Color32::from_rgb(255, 220, 60)
    };
    painter.text(
        egui::pos2(center.x, center.y + line_h * 1.1),
        egui::Align2::CENTER_CENTER,
        if pressed_keys.is_empty() {
            "---"
        } else {
            &keys_str
        },
        egui::FontId::proportional(line_h * 0.9),
        keys_color,
    );

    // Player label
    painter.text(
        egui::pos2(center.x, center.y + line_h * 2.2),
        egui::Align2::CENTER_CENTER,
        player,
        egui::FontId::proportional(label_size),
        egui::Color32::from_gray(130),
    );
}

fn polar(center: egui::Pos2, angle: f32, r: f32) -> egui::Pos2 {
    center + egui::vec2(angle.cos() * r, angle.sin() * r)
}

fn key_is_down(vk: c_int) -> bool {
    unsafe { winapi::um::winuser::GetAsyncKeyState(vk) as u16 & 0x8000 != 0 }
}

fn vk_label(vk: c_int) -> String {
    match vk {
        0x30..=0x39 => ((b'0' + (vk - 0x30) as u8) as char).to_string(),
        0x41..=0x5A => ((b'A' + (vk - 0x41) as u8) as char).to_string(),
        0x60..=0x69 => format!("N{}", vk - 0x60),
        0x70 => "F1".to_string(),
        0x71 => "F2".to_string(),
        0x72 => "F3".to_string(),
        0x73 => "F4".to_string(),
        0x74 => "F5".to_string(),
        0x75 => "F6".to_string(),
        0x76 => "F7".to_string(),
        0x77 => "F8".to_string(),
        0x78 => "F9".to_string(),
        0x79 => "F10".to_string(),
        0x7A => "F11".to_string(),
        0x7B => "F12".to_string(),
        0x20 => "Spc".to_string(),
        0x0D => "Ent".to_string(),
        0x1B => "Esc".to_string(),
        0x08 => "Bsp".to_string(),
        0x09 => "Tab".to_string(),
        _ => format!("{:#04X}", vk),
    }
}

fn key_bind_row(ui: &mut egui::Ui, label: &str, vk: &mut c_int, w: &mut ConfigWidgets<'_>) {
    ui.label(label);
    let mut text = format!("0x{:02X}", vk);
    if ui
        .add(egui::TextEdit::singleline(&mut text).desired_width(55.0))
        .changed()
    {
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
