mod components;
mod console;
pub mod monitors;
mod panels;
pub mod state;

pub use state::State;

use crate::config::Config;
use crate::runtime::{ModuleName, ModuleRuntime};
use crate::{error::Result, gui::panels::show_panel};
use anyhow::Context;
use eframe::egui::{self, Layout, ViewportCommand};
use std::collections::HashSet;

const SINGLE_SCREEN_ASPECT: f32 = 9.0 / 16.0;
const DUAL_SCREEN_ASPECT: f32 = 9.0 / 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavAction {
    None,
    SetTab(Tab),
    ToggleDualScreen,
    Apply,
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    #[cfg(feature = "touch")]
    Touch,
    #[cfg(feature = "jvs")]
    Jvs,
    #[cfg(feature = "reader")]
    Reader,
    Config,
}

impl Tab {
    const ALL: &'static [(Tab, &'static str)] = &[
        #[cfg(feature = "touch")]
        (Tab::Touch, "Touch"),
        #[cfg(feature = "jvs")]
        (Tab::Jvs, "JVS"),
        #[cfg(feature = "reader")]
        (Tab::Reader, "Reader"),
        (Tab::Config, "Config"),
    ];
}

pub fn run(runtime: ModuleRuntime) -> Result<()> {
    runtime.shared_state().gui.set_active(true);
    let monitors = monitors::get_all_monitors();
    let primary = monitors
        .get_primary()
        .context("No primary monitor was found")?;

    let monitor_height = primary.height() as f32;
    let initial_width = monitor_height * SINGLE_SCREEN_ASPECT;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_fullscreen(true)
            .with_inner_size(egui::vec2(initial_width, monitor_height)),
        vsync: true,
        ..Default::default()
    };

    let config_editor = panels::config::ConfigEditor::new(runtime.config());
    let clean_config = runtime.config().clone();

    let app = App {
        runtime,
        clean_config,
        active_tab: Tab::ALL[0].0,
        config_editor,
        config_needs_reload: false,
        pending_restarts: HashSet::new(),
        #[cfg(feature = "touch")]
        touch_editor: panels::touch::TouchEditorState::default(),
        dual_screen: false,
        monitor_height,
        expected_size: egui::vec2(initial_width, monitor_height),
    };

    eframe::run_native("MaiFinaleToDX", options, Box::new(|_cc| Ok(Box::new(app))))
        .map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}

struct App {
    runtime: ModuleRuntime,
    clean_config: Config,
    active_tab: Tab,
    config_editor: panels::config::ConfigEditor,
    config_needs_reload: bool,
    pending_restarts: HashSet<ModuleName>,
    #[cfg(feature = "touch")]
    touch_editor: panels::touch::TouchEditorState,
    dual_screen: bool,
    monitor_height: f32,
    expected_size: egui::Vec2,
}

impl App {
    fn target_aspect_ratio(&self) -> f32 {
        if self.dual_screen {
            DUAL_SCREEN_ASPECT
        } else {
            SINGLE_SCREEN_ASPECT
        }
    }

    fn any_dirty(&self) -> bool {
        !self.pending_restarts.is_empty() || self.config_editor.dirty
    }

    fn apply_all(&mut self) {
        if self.config_editor.dirty {
            match toml_edit::de::from_str::<Config>(&self.config_editor.config_text) {
                Ok(new_config) => match new_config.save("./config.toml") {
                    Ok(()) => {
                        self.runtime.update_config(new_config);
                        self.runtime.stop_all();
                        self.runtime.start_all();
                        self.clean_config = self.runtime.config().clone();
                        self.config_editor.dirty = false;
                        self.pending_restarts.clear();
                        self.config_needs_reload = true;
                        tracing::info!("Config applied and all modules restarted from GUI");
                    }
                    Err(e) => tracing::error!("Failed to save config: {}", e),
                },
                Err(e) => tracing::error!("Invalid TOML: {}", e),
            }
            return;
        }

        if let Err(e) = self.runtime.config().save("./config.toml") {
            tracing::error!("Failed to save config: {}", e);
            return;
        }

        for module in self.pending_restarts.drain() {
            self.runtime.restart_module(module);
        }
        self.clean_config = self.runtime.config().clone();
        self.config_needs_reload = true;
    }

    fn discard_all(&mut self) {
        self.pending_restarts.clear();
        self.runtime.update_config(self.clean_config.clone());
        self.config_editor.reload_from(&self.clean_config);
    }

    fn handle_nav_action(&mut self, ctx: &egui::Context, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::SetTab(tab) => self.active_tab = tab,
            NavAction::ToggleDualScreen => {
                self.dual_screen = !self.dual_screen;
                let aspect = self.target_aspect_ratio();
                let new_width = self.monitor_height * aspect;
                self.expected_size = egui::vec2(new_width, self.monitor_height);
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(self.expected_size));
            }
            NavAction::Apply => self.apply_all(),
            NavAction::Discard => self.discard_all(),
        }
    }
}

fn render_nav_bar(ui: &mut egui::Ui, active_tab: Tab, dual_screen: bool, any_dirty: bool) -> NavAction {
    let mut action = NavAction::None;
    ui.horizontal(|ui| {
        for &(tab, label) in Tab::ALL {
            if ui.selectable_label(active_tab == tab, label).clicked() {
                action = NavAction::SetTab(tab);
            }
        }
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let label = if dual_screen { "Collapse" } else { "Expand" };
            if ui.button(label).clicked() {
                action = NavAction::ToggleDualScreen;
            }
            if any_dirty {
                if ui.button("Apply Unsaved Changes").clicked() {
                    action = NavAction::Apply;
                }
                if ui.button("Dismiss").clicked() {
                    action = NavAction::Discard;
                }
            }
        });
    });
    action
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let actual_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
        if let Some(actual) = actual_size {
            let diff = (actual - self.expected_size).abs();
            if diff.x > 1.0 || diff.y > 1.0 {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(self.expected_size));
            }
        }

        let is_visible = !ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        self.runtime.shared_state().gui.set_active(is_visible);

        egui::TopBottomPanel::top("nav_panel")
            .resizable(false)
            .show(ctx, |ui| {
                let any_dirty = self.any_dirty();
                let active_tab = self.active_tab;
                let dual_screen = self.dual_screen;

                let action = if dual_screen {
                    ui.columns(2, |cols| {
                        let a = render_nav_bar(&mut cols[0], active_tab, dual_screen, any_dirty);
                        let b = render_nav_bar(&mut cols[1], active_tab, dual_screen, any_dirty);
                        if !matches!(a, NavAction::None) { a } else { b }
                    })
                } else {
                    render_nav_bar(ui, active_tab, dual_screen, any_dirty)
                };

                self.handle_nav_action(ctx, action);
            });

        let dual_screen = self.dual_screen;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(0).fill(ctx.style().visuals.panel_fill))
            .show(ctx, |ui| {
                match self.active_tab {
                    #[cfg(feature = "touch")]
                    Tab::Touch => {
                        let mut panel = panels::touch::Touch::new(
                            &mut self.runtime,
                            &mut self.touch_editor,
                            &mut self.pending_restarts,
                        );
                        show_panel(ui, &mut panel, dual_screen);
                    }
                    #[cfg(feature = "jvs")]
                    Tab::Jvs => {
                        let mut panel = panels::jvs::Jvs::new(
                            &mut self.runtime,
                            &mut self.pending_restarts,
                        );
                        show_panel(ui, &mut panel, dual_screen);
                    }
                    #[cfg(feature = "reader")]
                    Tab::Reader => {
                        let mut panel = panels::reader::Reader::new(
                            &mut self.runtime,
                            &mut self.pending_restarts,
                        );
                        show_panel(ui, &mut panel, dual_screen);
                    }
                    Tab::Config => {
                        let mut panel = panels::config::ConfigPanel::new(
                            &mut self.config_editor,
                            &mut self.runtime,
                        );
                        show_panel(ui, &mut panel, dual_screen);
                    }
                };
            });

        if self.config_needs_reload {
            self.config_editor.reload_from(self.runtime.config());
            self.config_needs_reload = false;
        }

        if is_visible {
            ctx.request_repaint();
        }
    }
}
