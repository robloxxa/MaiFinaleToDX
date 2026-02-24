use eframe::egui::{self, Rect, Ui, UiBuilder};

pub struct PanelLayout {
    pub left_header: Ui,
    pub left_body: Ui,
    pub right_header: Option<Ui>,
    pub right_body: Option<Ui>,
}

pub fn split(ui: &mut Ui, dual_screen: bool) -> PanelLayout {
    let rect = ui.available_rect_before_wrap();
    let layout = egui::Layout::top_down(egui::Align::LEFT);

    ui.allocate_rect(rect, egui::Sense::hover());

    let (left_rect, right_rect) = if dual_screen {
        let mid = rect.center().x;
        (
            Rect::from_min_max(rect.min, egui::pos2(mid, rect.max.y)),
            Some(Rect::from_min_max(
                egui::pos2(mid, rect.min.y),
                rect.max,
            )),
        )
    } else {
        (rect, None)
    };

    let (lh, lb) = split_header_body(left_rect);
    let left_header = ui.new_child(UiBuilder::new().max_rect(lh).layout(layout));
    let left_body = ui.new_child(UiBuilder::new().max_rect(lb).layout(layout));

    let (right_header, right_body) = match right_rect {
        Some(rr) => {
            let (rh, rb) = split_header_body(rr);
            (
                Some(ui.new_child(UiBuilder::new().max_rect(rh).layout(layout))),
                Some(ui.new_child(UiBuilder::new().max_rect(rb).layout(layout))),
            )
        }
        None => (None, None),
    };

    PanelLayout {
        left_header,
        left_body,
        right_header,
        right_body,
    }
}

const HEADER_ASPECT: f32 = 780.0 / 325.0;

fn split_header_body(rect: Rect) -> (Rect, Rect) {
    let w = rect.width();

    // Body: 1:1 square pinned to the bottom
    let body_side = w.min(rect.height());
    let body = Rect::from_min_size(
        egui::pos2(rect.min.x, rect.max.y - body_side),
        egui::vec2(body_side, body_side),
    );

    // Header: 780:325 aspect ratio, pinned to the top
    let header_h = (w / HEADER_ASPECT).min(body.min.y - rect.min.y);
    let header = Rect::from_min_size(rect.min, egui::vec2(w, header_h));

    (header, body)
}
