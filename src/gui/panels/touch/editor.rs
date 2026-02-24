use crate::config::touch::finale;
use crate::config::touch::ZoneId;
use crate::runtime::{ModuleName, ModuleRuntime};
use std::collections::HashSet;
use eframe::egui::{self, Color32};
use eframe::epaint::{Mesh, PathShape, PathStroke};
use std::f32::consts::PI;
use std::time::Duration;

use super::zone_shape::{point_in_polygon, polar};
use super::Player;

#[derive(Default)]
pub struct TouchEditorState {
    pub open_zone: Option<(Player, ZoneId)>,
}

const FINALE_A_AREAS: [finale::Area; 8] = [
    finale::A1, finale::A2, finale::A3, finale::A4,
    finale::A5, finale::A6, finale::A7, finale::A8,
];
const FINALE_B_AREAS: [finale::Area; 8] = [
    finale::B1, finale::B2, finale::B3, finale::B4,
    finale::B5, finale::B6, finale::B7, finale::B8,
];

fn toggle_finale_area(activate_on: &mut finale::AreaVec, area: finale::Area, selected: bool) {
    if selected {
        activate_on.0.retain(|a| a.name != area.name);
    } else {
        activate_on.0.push(area);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DxOverlay {
    A(u8),
    B(u8),
    C1,
    C2,
    D(u8),
    E(u8),
}

fn draw_finale_selector_circle(
    ui: &mut egui::Ui,
    activate_on: &mut finale::AreaVec,
    overlay: Option<DxOverlay>,
    size: f32,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = (size - 2.0) / 2.0;
    let pointer = ui.ctx().input(|i| i.pointer.clone());
    let mut changed = false;

    let selected_fill = Color32::from_rgba_unmultiplied(80, 230, 80, 160);
    let unselected_fill = Color32::from_rgba_unmultiplied(80, 80, 80, 80);
    let overlay_stroke = PathStroke::new(2.5, Color32::from_rgb(0, 220, 255));

    // C zone — center octagon (Finale toggle)
    {
        let selected = activate_on.0.iter().any(|a| a.name == finale::C.name);
        let oct_r = radius * 0.18;
        let v: Vec<egui::Pos2> = (0..8)
            .map(|k| polar(center, -PI / 2.0 - PI / 8.0 + k as f32 * PI / 4.0, oct_r))
            .collect();
        let hovered = pointer
            .hover_pos()
            .map_or(false, |p| point_in_polygon(p, &v));
        let fill = if selected { selected_fill } else { unselected_fill };

        painter.add(egui::Shape::Path(PathShape {
            points: v.clone(),
            closed: true,
            fill,
            stroke: PathStroke::new(1.5, Color32::WHITE),
        }));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "C",
            egui::FontId::proportional((radius * 0.08).clamp(6.0, 12.0)),
            Color32::WHITE,
        );

        if hovered && pointer.button_clicked(egui::PointerButton::Primary) {
            toggle_finale_area(activate_on, finale::C, selected);
            changed = true;
        }

        // C1/C2 overlay
        if matches!(overlay, Some(DxOverlay::C1) | Some(DxOverlay::C2)) {
            let gap = oct_r * 0.1;
            let y_top = center.y - (PI / 8.0_f32).cos() * oct_r;
            let y_bot = center.y + (PI / 8.0_f32).cos() * oct_r;

            let poly = if overlay == Some(DxOverlay::C1) {
                let top_l = egui::pos2(center.x - gap, y_top);
                let bot_l = egui::pos2(center.x - gap, y_bot);
                vec![top_l, v[0], v[7], v[6], v[5], bot_l]
            } else {
                let top_r = egui::pos2(center.x + gap, y_top);
                let bot_r = egui::pos2(center.x + gap, y_bot);
                vec![top_r, v[1], v[2], v[3], v[4], bot_r]
            };
            painter.add(egui::Shape::Path(PathShape {
                points: poly,
                closed: true,
                fill: Color32::TRANSPARENT,
                stroke: overlay_stroke.clone(),
            }));
        }
    }

    for i in 0..8usize {
        let d_angle = -PI / 2.0 + i as f32 * PI / 4.0;
        let a_angle = d_angle + PI / 8.0;
        let num = (i + 1) as u8;

        // A zone — outer arc segment (Finale toggle)
        {
            let area = FINALE_A_AREAS[i];
            let selected = activate_on.0.iter().any(|a| a.name == area.name);
            let fill = if selected { selected_fill } else { unselected_fill };

            let half_outer = 14.0_f32.to_radians();
            let half_inner = 7.0_f32.to_radians();
            let outer_r = radius;
            let mid_r = radius * 0.65;
            let inner_r = radius * 0.60;

            const ARC_STEPS: usize = 8;
            let arc: Vec<egui::Pos2> = (0..=ARC_STEPS)
                .map(|s| {
                    polar(
                        center,
                        (a_angle - half_outer) + s as f32 / ARC_STEPS as f32 * 2.0 * half_outer,
                        outer_r,
                    )
                })
                .collect();

            let tr = polar(center, a_angle + half_outer, mid_r);
            let br = polar(center, a_angle + half_inner, inner_r);
            let bl = polar(center, a_angle - half_inner, inner_r);
            let tl = polar(center, a_angle - half_outer, mid_r);

            let polygon: Vec<egui::Pos2> =
                arc.iter().cloned().chain([tr, br, bl, tl]).collect();
            let hovered = pointer
                .hover_pos()
                .map_or(false, |p| point_in_polygon(p, &polygon));

            let mut mesh = Mesh::default();
            for &pos in &polygon {
                mesh.colored_vertex(pos, fill);
            }
            let n = mesh.vertices.len() as u32;
            for j in 1..n - 1 {
                mesh.add_triangle(0, j, j + 1);
            }
            painter.add(egui::Shape::Mesh(mesh.into()));

            let a_stroke = if overlay == Some(DxOverlay::A(num)) {
                overlay_stroke.clone()
            } else {
                PathStroke::new(1.5, Color32::WHITE)
            };
            painter.add(egui::Shape::Path(PathShape {
                points: polygon,
                closed: true,
                fill: Color32::TRANSPARENT,
                stroke: a_stroke,
            }));

            painter.text(
                polar(center, a_angle, radius * 0.80),
                egui::Align2::CENTER_CENTER,
                &format!("A{num}"),
                egui::FontId::proportional((radius * 0.09).clamp(7.0, 14.0)),
                Color32::WHITE,
            );

            if hovered && pointer.button_clicked(egui::PointerButton::Primary) {
                toggle_finale_area(activate_on, area, selected);
                changed = true;
            }
        }

        // B zone — inner segment (Finale toggle)
        {
            let area = FINALE_B_AREAS[i];
            let selected = activate_on.0.iter().any(|a| a.name == area.name);
            let fill = if selected { selected_fill } else { unselected_fill };

            let half_a = 10.0_f32.to_radians();
            let half_b = 20.0_f32.to_radians();
            let outer_r = radius * 0.54;
            let mid_r = radius * 0.43;
            let lower_r = radius * 0.36;
            let tip_r = radius * 0.28;

            let points = vec![
                polar(center, a_angle - half_a, outer_r),
                polar(center, a_angle + half_a, outer_r),
                polar(center, a_angle + half_b, mid_r),
                polar(center, a_angle + half_b, lower_r),
                polar(center, a_angle, tip_r),
                polar(center, a_angle - half_b, lower_r),
                polar(center, a_angle - half_b, mid_r),
            ];

            let hovered = pointer
                .hover_pos()
                .map_or(false, |p| point_in_polygon(p, &points));

            let mut mesh = Mesh::default();
            for &pos in &points {
                mesh.colored_vertex(pos, fill);
            }
            for j in 1..6_u32 {
                mesh.add_triangle(0, j, j + 1);
            }
            painter.add(egui::Shape::Mesh(mesh.into()));

            let b_stroke = if overlay == Some(DxOverlay::B(num)) {
                overlay_stroke.clone()
            } else {
                PathStroke::new(1.5, Color32::WHITE)
            };
            painter.add(egui::Shape::Path(PathShape {
                points,
                closed: true,
                fill: Color32::TRANSPARENT,
                stroke: b_stroke,
            }));

            painter.text(
                polar(center, a_angle, radius * 0.41),
                egui::Align2::CENTER_CENTER,
                &format!("B{num}"),
                egui::FontId::proportional((radius * 0.08).clamp(6.0, 12.0)),
                Color32::WHITE,
            );

            if hovered && pointer.button_clicked(egui::PointerButton::Primary) {
                toggle_finale_area(activate_on, area, selected);
                changed = true;
            }
        }

        // D zone overlay (outline only, not interactive)
        if overlay == Some(DxOverlay::D(num)) {
            let half = 7.0_f32.to_radians();
            let outer_r = radius;
            let side_r = radius * 0.66;
            let tip_r = radius * 0.74;

            const ARC_STEPS: usize = 6;
            let arc: Vec<egui::Pos2> = (0..=ARC_STEPS)
                .map(|s| {
                    polar(
                        center,
                        (d_angle - half) + s as f32 / ARC_STEPS as f32 * 2.0 * half,
                        outer_r,
                    )
                })
                .collect();

            let pt_c = polar(center, d_angle + half, side_r);
            let pt_d = polar(center, d_angle, tip_r);
            let pt_e = polar(center, d_angle - half, side_r);

            let polygon: Vec<egui::Pos2> =
                arc.iter().cloned().chain([pt_c, pt_d, pt_e]).collect();
            painter.add(egui::Shape::Path(PathShape {
                points: polygon,
                closed: true,
                fill: Color32::TRANSPARENT,
                stroke: overlay_stroke.clone(),
            }));
            painter.text(
                polar(center, d_angle, radius * 0.84),
                egui::Align2::CENTER_CENTER,
                &format!("D{num}"),
                egui::FontId::proportional((radius * 0.07).clamp(6.0, 11.0)),
                Color32::from_rgb(0, 220, 255),
            );
        }

        // E zone overlay (outline only, not interactive)
        if overlay == Some(DxOverlay::E(num)) {
            let inner_r = radius * 0.58;
            let sz = radius * 0.12;
            let zone_center = polar(center, d_angle, inner_r);
            let outward = egui::vec2(d_angle.cos(), d_angle.sin());
            let tangent = egui::vec2(-d_angle.sin(), d_angle.cos());

            let points = vec![
                zone_center + outward * sz,
                zone_center + tangent * sz,
                zone_center - outward * sz,
                zone_center - tangent * sz,
            ];
            painter.add(egui::Shape::Path(PathShape {
                points,
                closed: true,
                fill: Color32::TRANSPARENT,
                stroke: overlay_stroke.clone(),
            }));
            painter.text(
                zone_center,
                egui::Align2::CENTER_CENTER,
                &format!("E{num}"),
                egui::FontId::proportional((radius * 0.07).clamp(6.0, 11.0)),
                Color32::from_rgb(0, 220, 255),
            );
        }
    }

    changed
}

pub(super) fn show_zone_editor(
    ctx: &egui::Context,
    runtime: &mut ModuleRuntime,
    editor: &mut TouchEditorState,
    pending_restarts: &mut HashSet<ModuleName>,
) {
    let Some((player, ref zone_id)) = editor.open_zone.clone() else {
        return;
    };
    let mut open = true;

    let pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or_default());

    egui::Window::new(format!("{} {}", player, zone_id))
        .id(egui::Id::new("touch_zone_editor"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_pos(pos)
        .show(ctx, |ui| {
            let config = runtime.config_mut();

            let threshold = match player {
                Player::P1 => &mut config.touch.finale.p1_threshold,
                Player::P2 => &mut config.touch.finale.p2_threshold,
            };
            if let Some(val) = threshold.field_mut(&zone_id) {
                ui.horizontal(|ui| {
                    ui.label("Threshold:");
                    if ui.add(egui::Slider::new(val, 0..=255)).changed() {
                        pending_restarts.insert(ModuleName::TouchFinale);
                    }
                });
            }

            let mapping = match player {
                Player::P1 => &mut config.touch.dx.p1_mapping,
                Player::P2 => &mut config.touch.dx.p2_mapping,
            };
            for (idx, area) in mapping.fields_mut(&zone_id).into_iter().enumerate() {
                let overlay = match &zone_id {
                    ZoneId::A(n) => Some(DxOverlay::A(*n)),
                    ZoneId::B(n) => Some(DxOverlay::B(*n)),
                    ZoneId::C => Some(if idx == 0 { DxOverlay::C1 } else { DxOverlay::C2 }),
                    ZoneId::D(n) => Some(DxOverlay::D(*n)),
                    ZoneId::E(n) => Some(DxOverlay::E(*n)),
                };
                ui.separator();
                ui.label("Activate on:");
                let circle_size = 200.0;
                if draw_finale_selector_circle(ui, &mut area.activate_on, overlay, circle_size) {
                    pending_restarts.insert(ModuleName::TouchDeluxe(1));
                    pending_restarts.insert(ModuleName::TouchDeluxe(2));
                }

                ui.horizontal(|ui| {
                    ui.label("Deactivate after (ms):");
                    let mut ms = area.deactivate_after_ms.as_millis() as u64;
                    if ui.add(egui::DragValue::new(&mut ms)).changed() {
                        area.deactivate_after_ms = Duration::from_millis(ms);
                        pending_restarts.insert(ModuleName::TouchDeluxe(1));
                        pending_restarts.insert(ModuleName::TouchDeluxe(2));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Reactivate after (ms):");
                    let mut ms = area.reactivate_after_ms.as_millis() as u64;
                    if ui.add(egui::DragValue::new(&mut ms)).changed() {
                        area.reactivate_after_ms = Duration::from_millis(ms);
                        pending_restarts.insert(ModuleName::TouchDeluxe(1));
                        pending_restarts.insert(ModuleName::TouchDeluxe(2));
                    }
                });
            }
        });

    if !open {
        editor.open_zone = None;
    }
}
