use crate::gui::Render;
use anyhow::Context;
use eframe::egui;
use eframe::egui::Ui;
use tracing::Level;
use tracing_subscriber::filter::Targets;

#[derive(Default)]
pub struct TracerWindow {
    enabled: bool,
}

impl Render for TracerWindow {
    fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("Tracer")
            .default_width(280.0)
            .show(ctx, |ui| {
                if let Err(e) = self.render(ui) {
                    ui.code(e.to_string());
                }
            });
    }

    fn render(&mut self, ui: &mut Ui) -> anyhow::Result<()> {
        if ui.checkbox(&mut self.enabled, "Tracing").changed() {
            let reload = crate::statics::TRACER_RELOAD_HANDLE
                .get()
                .context("Unable to get handle")?;
            reload.modify(|layer| {
                *layer = Targets::new().with_target(
                    "tracer",
                    if self.enabled {
                        Level::TRACE
                    } else {
                        Level::ERROR
                    },
                );
            })?;
        }

        Ok(())
    }
}
