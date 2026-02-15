mod console;
mod panels;
mod components;

use crate::error::Result;
use crate::runtime::ModuleRuntime;
use eframe::egui;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Touch,
    Jvs,
    Reader,
    Config,
    Emulation,
    Log,
}

pub fn run(runtime: ModuleRuntime) -> Result<()> {
    runtime.shared_state().gui_active.store(true, Ordering::Relaxed);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    let log_buffer = panels::log_viewer::LogBuffer::new();
    let config_editor = panels::config::ConfigEditor::new(runtime.config());
    let emulation_state = panels::emulation::EmulationState::default();

    let app = App {
        runtime,
        active_tab: Tab::Touch,
        log_buffer,
        config_editor,
        emulation_state,
    };

    eframe::run_native(
        "MaiFinaleToDX",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}

struct App {
    runtime: ModuleRuntime,
    active_tab: Tab,
    log_buffer: panels::log_viewer::LogBuffer,
    config_editor: panels::config::ConfigEditor,
    emulation_state: panels::emulation::EmulationState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_visible = !ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        self.runtime
            .shared_state()
            .gui_active
            .store(is_visible, Ordering::Relaxed);

        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(120.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("MaiDX");
                });
                ui.separator();

                let tabs = [
                    (Tab::Touch, "Touch"),
                    (Tab::Jvs, "JVS"),
                    (Tab::Reader, "Reader"),
                    (Tab::Config, "Config"),
                    (Tab::Emulation, "Emulation"),
                    (Tab::Log, "Log"),
                ];

                for (tab, label) in &tabs {
                    if ui
                        .selectable_label(self.active_tab == *tab, *label)
                        .clicked()
                    {
                        self.active_tab = *tab;
                    }
                }

                ui.separator();

                let mut console_visible = console::is_console_visible();
                if ui.checkbox(&mut console_visible, "Show Console").changed() {
                    console::set_console_visible(console_visible);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            // Tab::Status => panels::status::show(ui, &mut self.runtime),
            Tab::Touch => panels::touch::show(ui, &mut self.runtime),
            Tab::Jvs => panels::jvs::show(ui, &mut self.runtime),
            Tab::Reader => panels::reader::show(ui, &mut self.runtime),
            Tab::Config => {
                panels::config::show(ui, &mut self.config_editor, &mut self.runtime)
            }
            Tab::Emulation => panels::emulation::show(ui, &mut self.emulation_state),
            Tab::Log => panels::log_viewer::show(ui, &self.log_buffer),
        });

        if is_visible {
            ctx.request_repaint();
        }
    }
}
