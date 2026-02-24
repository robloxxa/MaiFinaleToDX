use eframe::egui::{self, Color32, Pos2};
use eframe::epaint::{Mesh, PathShape, PathStroke};

pub(super) fn polar(center: Pos2, angle: f32, r: f32) -> Pos2 {
    center + egui::vec2(angle.cos() * r, angle.sin() * r)
}

// Ray-casting point-in-polygon test. Works for any simple polygon.
pub(super) fn point_in_polygon(pos: Pos2, polygon: &[Pos2]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[j];
        if ((a.y > pos.y) != (b.y > pos.y))
            && pos.x < (b.x - a.x) * (pos.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub(super) fn finale_zone_stroke(finale_active: bool) -> PathStroke {
    if finale_active {
        PathStroke::new(2.0, Color32::from_rgba_unmultiplied(80, 230, 80, 255))
    } else {
        PathStroke::new(1.5, Color32::TRANSPARENT)
    }
}

pub(super) fn dx_zone_stroke(finale_active: bool) -> PathStroke {
    if finale_active {
        PathStroke::new(2.0, Color32::from_rgba_unmultiplied(80, 230, 80, 255))
    } else {
        PathStroke::new(1.5, Color32::WHITE)
    }
}

pub(super) fn zone_fill(dx_active: bool) -> Color32 {
    if dx_active {
        Color32::from_rgba_unmultiplied(220, 60, 60, 220)
    } else {
        Color32::TRANSPARENT
    }
}

pub struct ZoneResponse {
    pub left_down: bool,
    pub secondary_clicked: bool,
}

pub struct ZoneShape {
    points: Vec<Pos2>,
    dx_active: bool,
    // Some(hw_state) enables GUI press highlighting (A/B zones); None = fixed white stroke (D/E).
    finale_active: Option<bool>,
    label: Option<(Pos2, String, f32)>,
}

impl ZoneShape {
    pub fn new(points: Vec<Pos2>) -> Self {
        Self {
            points,
            dx_active: false,
            finale_active: None,
            label: None,
        }
    }

    pub fn dx_active(mut self, active: bool) -> Self {
        self.dx_active = active;
        self
    }

    pub fn finale_active(mut self, active: bool) -> Self {
        self.finale_active = Some(active);
        self
    }

    pub fn label(mut self, pos: Pos2, text: impl Into<String>, font_size: f32) -> Self {
        self.label = Some((pos, text.into(), font_size));
        self
    }

    pub fn show(self, painter: &egui::Painter, pointer: &egui::PointerState) -> ZoneResponse {
        let hovered = pointer
            .hover_pos()
            .map_or(false, |p| point_in_polygon(p, &self.points));
        let left_down = hovered && pointer.button_down(egui::PointerButton::Primary);
        let effective_finale_active = match self.finale_active {
            Some(hw) => hw || left_down,
            None => false,
        };

        let fill = zone_fill(self.dx_active);

        // Fan triangulation from vertex 0 — valid for all convex-ish zone shapes.
        let mut mesh = Mesh::default();
        for &pos in &self.points {
            mesh.colored_vertex(pos, fill);
        }
        let n = mesh.vertices.len() as u32;
        for i in 1..n - 1 {
            mesh.add_triangle(0, i, i + 1);
        }
        painter.add(egui::Shape::Mesh(mesh.into()));

        painter.add(egui::Shape::Path(PathShape {
            points: self.points,
            closed: true,
            fill: Color32::TRANSPARENT,
            stroke: dx_zone_stroke(effective_finale_active),
        }));

        if let Some((pos, text, font_size)) = self.label {
            painter.text(
                pos,
                egui::Align2::CENTER_CENTER,
                &text,
                egui::FontId::proportional(font_size),
                Color32::WHITE,
            );
        }

        ZoneResponse {
            left_down,
            secondary_clicked: hovered && pointer.button_clicked(egui::PointerButton::Secondary),
        }
    }
}
