use eframe::egui::{self, Color32};
use eframe::epaint::PathShape;
use std::f32::consts::PI;

use crate::config::touch::ZoneId;
use super::editor::TouchEditorState;
use super::zone_shape::{
    dx_zone_stroke, finale_zone_stroke, point_in_polygon, polar, zone_fill, ZoneShape,
};
use super::Player;

// (byte_index, bit_mask) for each zone in the 9-byte DX packet, zone order 1..=8.
const DX_A: [(usize, u8); 8] = [
    (1, 1),
    (1, 2),
    (1, 4),
    (1, 8),
    (1, 16),
    (2, 1),
    (2, 2),
    (2, 4),
];
const DX_B: [(usize, u8); 8] = [
    (2, 8),
    (2, 16),
    (3, 1),
    (3, 2),
    (3, 4),
    (3, 8),
    (3, 16),
    (4, 1),
];
const DX_D: [(usize, u8); 8] = [
    (4, 8),
    (4, 16),
    (5, 1),
    (5, 2),
    (5, 4),
    (5, 8),
    (5, 16),
    (6, 1),
];
const DX_E: [(usize, u8); 8] = [
    (6, 2),
    (6, 4),
    (6, 8),
    (6, 16),
    (7, 1),
    (7, 2),
    (7, 4),
    (7, 8),
];
const DX_C1: (usize, u8) = (4, 2);
const DX_C2: (usize, u8) = (4, 4);

// (byte_index, bit_number) for each zone in the 4-byte Finale raw state, zone order 1..=8.
// Bit mask = 1 << bit_number.
const FINALE_A: [(usize, u8); 8] = [
    (0, 0),
    (0, 2),
    (1, 0),
    (1, 2),
    (2, 0),
    (2, 2),
    (3, 0),
    (3, 2),
];
const FINALE_B: [(usize, u8); 8] = [
    (0, 1),
    (0, 3),
    (1, 1),
    (1, 3),
    (2, 1),
    (2, 3),
    (3, 1),
    (3, 3),
];
const FINALE_C: (usize, u8) = (3, 4);

fn dx_bit(dx_raw: &[u8; 9], byte: usize, mask: u8) -> bool {
    dx_raw[byte] & mask != 0
}

fn finale_read(raw: &[u8; 4], pos: usize, bit: u8) -> bool {
    (raw[pos] >> bit) & 1 != 0
}

fn a_zone_points(center: egui::Pos2, radius: f32, angle: f32) -> Vec<egui::Pos2> {
    let half_outer = 14.0_f32.to_radians();
    let half_inner = 7.0_f32.to_radians();
    let outer_r = radius;
    let mid_r = radius * 0.65;
    let inner_r = radius * 0.60;

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

    arc.into_iter().chain([tr, br, bl, tl]).collect()
}

fn b_zone_points(center: egui::Pos2, radius: f32, angle: f32) -> Vec<egui::Pos2> {
    let half_a = 10.0_f32.to_radians();
    let half_b = 20.0_f32.to_radians();
    let outer_r = radius * 0.54;
    let mid_r = radius * 0.43;
    let lower_r = radius * 0.36;
    let tip_r = radius * 0.28;

    vec![
        polar(center, angle - half_a, outer_r), // outer-left
        polar(center, angle + half_a, outer_r), // outer-right
        polar(center, angle + half_b, mid_r),   // shoulder-right (radial step)
        polar(center, angle + half_b, lower_r), // lower-right
        polar(center, angle, tip_r),            // tip toward center
        polar(center, angle - half_b, lower_r), // lower-left
        polar(center, angle - half_b, mid_r),   // shoulder-left (radial step)
    ]
}

fn d_zone_points(center: egui::Pos2, radius: f32, angle: f32) -> Vec<egui::Pos2> {
    let half = 7.0_f32.to_radians();
    let outer_r = radius;
    let side_r = radius * 0.66;
    let tip_r = radius * 0.74;

    const ARC_STEPS: usize = 6;
    let arc: Vec<egui::Pos2> = (0..=ARC_STEPS)
        .map(|i| {
            polar(
                center,
                (angle - half) + i as f32 / ARC_STEPS as f32 * 2.0 * half,
                outer_r,
            )
        })
        .collect();

    let pt_c = polar(center, angle + half, side_r);
    let pt_d = polar(center, angle, tip_r);
    let pt_e = polar(center, angle - half, side_r);

    arc.into_iter().chain([pt_c, pt_d, pt_e]).collect()
}

// Shape: rhombus (square rotated 45°) — one tip toward center, one away, two tips tangential.
// Uses Cartesian offsets to avoid unequal side lengths from angular distortion.
fn e_zone_points(center: egui::Pos2, radius: f32, angle: f32) -> Vec<egui::Pos2> {
    let inner_r = radius * 0.58;
    let size = radius * 0.12;
    let zone_center = polar(center, angle, inner_r);
    let outward = egui::vec2(angle.cos(), angle.sin());
    let tangent = egui::vec2(-angle.sin(), angle.cos());

    vec![
        zone_center + outward * size,  // outer tip
        zone_center + tangent * size,  // right tip
        zone_center - outward * size,  // inner tip
        zone_center - tangent * size,  // left tip
    ]
}

// Draws C1 (left) and C2 (right) as two visual halves of a single interactable zone.
// Hovering or clicking either half activates both as one unit.
// Returns (left_pressed, right_clicked).
fn draw_c_zones(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    pointer: &egui::PointerState,
    finale_hw: bool,
    c1_dx: bool,
    c2_dx: bool,
) -> (bool, bool) {
    let oct_r = radius * 0.18;
    let gap = oct_r * 0.1;

    // Rotated 22.5° so top/bottom edges are flat (horizontal), not pointed.
    // V0=top-left corner, V1=top-right corner, V2=right-top, V3=right-bot,
    // V4=bottom-right corner, V5=bottom-left corner, V6=left-bot, V7=left-top.
    let v: Vec<egui::Pos2> = (0..8)
        .map(|k| polar(center, -PI / 2.0 - PI / 8.0 + k as f32 * PI / 4.0, oct_r))
        .collect();

    // The flat top edge (V0-V1) and flat bottom edge (V4-V5) are symmetric about x=center.x,
    // so their midpoints lie exactly at center.x.
    let y_top = center.y - (PI / 8.0_f32).cos() * oct_r;
    let y_bot = center.y + (PI / 8.0_f32).cos() * oct_r;

    let top_l = egui::pos2(center.x - gap, y_top);
    let bot_l = egui::pos2(center.x - gap, y_bot);
    let top_r = egui::pos2(center.x + gap, y_top);
    let bot_r = egui::pos2(center.x + gap, y_bot);

    let poly_c1 = vec![top_l, v[0], v[7], v[6], v[5], bot_l];
    let poly_c2 = vec![top_r, v[1], v[2], v[3], v[4], bot_r];

    let hovered = pointer
        .hover_pos()
        .map_or(false, |p| point_in_polygon(p, &v));
    let left_pressed = hovered && pointer.button_down(egui::PointerButton::Primary);
    let finale_active = finale_hw || left_pressed;

    let stroke = dx_zone_stroke(finale_active);

    painter.add(egui::Shape::Path(PathShape {
        points: poly_c1,
        closed: true,
        fill: zone_fill(c1_dx),
        stroke: stroke.clone(),
    }));

    painter.add(egui::Shape::Path(PathShape {
        points: poly_c2,
        closed: true,
        fill: zone_fill(c2_dx),
        stroke,
    }));

    painter.add(egui::Shape::Path(PathShape {
        points: v,
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: finale_zone_stroke(finale_active),
    }));

    let font_size = (radius * 0.07).clamp(6.0, 11.0);
    painter.text(
        egui::pos2(center.x - oct_r * 0.5, center.y),
        egui::Align2::CENTER_CENTER,
        "C1",
        egui::FontId::proportional(font_size),
        Color32::WHITE,
    );
    painter.text(
        egui::pos2(center.x + oct_r * 0.5, center.y),
        egui::Align2::CENTER_CENTER,
        "C2",
        egui::FontId::proportional(font_size),
        Color32::WHITE,
    );

    let right_clicked = hovered && pointer.button_clicked(egui::PointerButton::Secondary);
    (left_pressed, right_clicked)
}

pub(super) fn draw_touch_circle(
    ui: &mut egui::Ui,
    finale_hw: &[u8; 4],
    dx_raw: &[u8; 9],
    size: f32,
    player: Player,
    editor: &mut TouchEditorState,
) -> [u8; 4] {
    let (rect, _response) =
        ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click_and_drag());

    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = (size - 2.0) / 2.0;

    let pointer = ui.ctx().input(|i| i.pointer.clone());

    let mut gui = [0u8; 4];

    let (c_clicked, c_right) = draw_c_zones(
        &painter,
        center,
        radius,
        &pointer,
        finale_read(finale_hw, FINALE_C.0, FINALE_C.1),
        dx_bit(dx_raw, DX_C1.0, DX_C1.1),
        dx_bit(dx_raw, DX_C2.0, DX_C2.1),
    );
    if c_clicked {
        gui[FINALE_C.0] |= 1 << FINALE_C.1;
    }
    if c_right {
        editor.open_zone = Some((player, ZoneId::C));
    }

    for i in 0..8usize {
        let d_angle = -PI / 2.0 + i as f32 * PI / 4.0;
        let a_angle = d_angle + PI / 8.0;
        let num = (i + 1) as u8;

        let a_resp = ZoneShape::new(a_zone_points(center, radius, a_angle))
            .dx_active(dx_bit(dx_raw, DX_A[i].0, DX_A[i].1))
            .finale_active(finale_read(finale_hw, FINALE_A[i].0, FINALE_A[i].1))
            .label(
                polar(center, a_angle, radius * 0.80),
                format!("A{num}"),
                (radius * 0.09).clamp(7.0, 14.0),
            )
            .show(&painter, &pointer);
        if a_resp.left_down {
            gui[FINALE_A[i].0] |= 1 << FINALE_A[i].1;
        }
        if a_resp.secondary_clicked {
            editor.open_zone = Some((player, ZoneId::A(num)));
        }

        let b_resp = ZoneShape::new(b_zone_points(center, radius, a_angle))
            .dx_active(dx_bit(dx_raw, DX_B[i].0, DX_B[i].1))
            .finale_active(finale_read(finale_hw, FINALE_B[i].0, FINALE_B[i].1))
            .label(
                polar(center, a_angle, radius * 0.41),
                format!("B{num}"),
                (radius * 0.08).clamp(6.0, 12.0),
            )
            .show(&painter, &pointer);
        if b_resp.left_down {
            gui[FINALE_B[i].0] |= 1 << FINALE_B[i].1;
        }
        if b_resp.secondary_clicked {
            editor.open_zone = Some((player, ZoneId::B(num)));
        }

        let d_resp = ZoneShape::new(d_zone_points(center, radius, d_angle))
            .dx_active(dx_bit(dx_raw, DX_D[i].0, DX_D[i].1))
            .label(
                polar(center, d_angle, radius * 0.84),
                format!("D{num}"),
                (radius * 0.09).clamp(7.0, 14.0),
            )
            .show(&painter, &pointer);
        if d_resp.secondary_clicked {
            editor.open_zone = Some((player, ZoneId::D(num)));
        }

        let e_zone_center = polar(center, d_angle, radius * 0.58);
        let e_resp = ZoneShape::new(e_zone_points(center, radius, d_angle))
            .dx_active(dx_bit(dx_raw, DX_E[i].0, DX_E[i].1))
            .label(
                e_zone_center,
                format!("E{num}"),
                (radius * 0.07).clamp(6.0, 11.0),
            )
            .show(&painter, &pointer);
        if e_resp.secondary_clicked {
            editor.open_zone = Some((player, ZoneId::E(num)));
        }
    }

    gui
}
