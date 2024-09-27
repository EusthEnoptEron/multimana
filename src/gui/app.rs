use std::sync::Arc;
use crate::gui::python_console::PythonConsole;
use crate::gui::{Panel, Render, Show};
use crate::utils::Loggable;
use anyhow::Context;
use eframe::egui;
use eframe::egui::Ui;
use tracing::Level;
use tracing_subscriber::filter::Targets;
use crate::statics::MESSAGE_BUS;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    tracing_enabled: bool,

    #[serde(skip)]
    panels: Vec<Box<dyn Panel>>,

    #[serde(skip)]
    panel_index: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            tracing_enabled: false,
            panels: vec![
                Box::new(PythonConsole::new())
            ],
            panel_index: 0,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app
    }

    fn tracing_button(&mut self, ui: &mut Ui) {
        if ui.checkbox(&mut self.tracing_enabled, "Enable Tracing").changed() {
            if let Ok(reload) = crate::statics::TRACER_RELOAD_HANDLE
                .get()
                .context("Unable to get handle") {
                reload.modify(|layer| {
                    *layer = Targets::new().with_target(
                        "tracer",
                        if self.tracing_enabled {
                            Level::TRACE
                        } else {
                            Level::ERROR
                        },
                    );
                }).and_log_if_err();
            } else {
                self.tracing_enabled = !self.tracing_enabled
            }
        }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;

                ui.menu_button("Settings", |ui| {
                    self.tracing_button(ui);
                });

                ui.separator();

                for (i, panel) in self.panels.iter().enumerate() {
                    if ui.selectable_label(i == self.panel_index, panel.get_name()).clicked() {
                        self.panel_index = i;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                if let Some(panel) = self.panels.get_mut(self.panel_index) {
                    if let Err(e) = panel.render(ui) {
                        ui.code(format!("Error while rendering: {:?}", e));
                    }
                } else {
                    ui.code("No panel selecte");
                }
            });
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}