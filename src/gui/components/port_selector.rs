use eframe::egui;
use serial2::SerialPort;
use std::time::{Duration, Instant};

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone)]
struct PortCache {
    ports: Vec<String>,
    refreshed_at: Instant,
}

fn get_ports(ctx: &egui::Context, force: bool) -> Vec<String> {
    let id = egui::Id::new("available_com_ports");

    if !force {
        let cached = ctx.data(|d| d.get_temp::<PortCache>(id));
        if let Some(c) = cached {
            if c.refreshed_at.elapsed() < REFRESH_INTERVAL {
                return c.ports;
            }
        }
    }

    let ports = SerialPort::available_ports()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let s = p.to_str()?.to_string();
            Some(s.strip_prefix(r"\\.\").unwrap_or(&s).to_string())
        })
        .collect::<Vec<_>>();

    ctx.data_mut(|d| {
        d.insert_temp(id, PortCache { ports: ports.clone(), refreshed_at: Instant::now() });
    });

    ports
}

/// A combo box populated with available COM ports with a refresh button.
/// Returns true if the port value changed.
pub fn port_combobox(ui: &mut egui::Ui, id_salt: impl std::hash::Hash, port: &mut String) -> bool {
    let ctx = ui.ctx().clone();
    let ports = get_ports(&ctx, false);

    let mut changed = false;
    let mut do_refresh = false;

    let selected_text = port.clone();
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(egui::Id::new(id_salt))
            .selected_text(selected_text.as_str())
            .width(70.0)
            .show_ui(ui, |ui| {
                for p in &ports {
                    changed |= ui.selectable_value(port, p.clone(), p).changed();
                }
                if !ports.contains(port) && !port.is_empty() {
                    let current = port.clone();
                    changed |= ui.selectable_value(port, current.clone(), current.as_str()).changed();
                }
            });

        if ui.small_button("↺").on_hover_text("Refresh port list").clicked() {
            do_refresh = true;
        }
    });

    if do_refresh {
        get_ports(&ctx, true);
    }

    changed
}

/// Like `port_combobox` but allows selecting None (no port).
/// Returns true if the value changed.
pub fn port_combobox_optional(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    port: &mut Option<String>,
) -> bool {
    let ctx = ui.ctx().clone();
    let ports = get_ports(&ctx, false);

    let mut changed = false;
    let mut do_refresh = false;
    let selected_text = port.as_deref().unwrap_or("None").to_string();

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(egui::Id::new(id_salt))
            .selected_text(selected_text.as_str())
            .width(70.0)
            .show_ui(ui, |ui| {
                changed |= ui.selectable_value(port, None, "None").changed();
                for p in &ports {
                    changed |= ui.selectable_value(port, Some(p.clone()), p).changed();
                }
                if let Some(current) = port.clone() {
                    if !ports.contains(&current) {
                        changed |= ui
                            .selectable_value(port, Some(current.clone()), current.as_str())
                            .changed();
                    }
                }
            });

        if ui.small_button("↺").on_hover_text("Refresh port list").clicked() {
            do_refresh = true;
        }
    });

    if do_refresh {
        get_ports(&ctx, true);
    }

    changed
}
