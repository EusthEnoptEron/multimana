use std::default::Default;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Context};
use crate::gui::{Panel, Render};
use eframe::egui;
use eframe::egui::{Align, Color32, Frame, Label, Layout, Margin, RichText, Ui};
use tracing::error;
use crate::statics::MESSAGE_BUS;
use crate::utils::{EventHandler, Loggable, Message, MessageBus};

#[derive(Default)]
struct OutputWrapper {
    output: Mutex<String>,
}

impl EventHandler for OutputWrapper {
    fn handle_evt(&self, e: &Message) -> anyhow::Result<()> {
        if let Message::PythonOutput { output } = e {
            if let Ok(mut output_mutex) = self.output.lock() {
                output_mutex.push_str(output.as_str());
                output_mutex.push_str("\n");
            } else {
                error!("Unable to lock output");
            }
        }

        Ok(())
    }
}

pub struct PythonConsole {
    code: String,
    eval_code: String,
    output: Arc<OutputWrapper>,
}

impl PythonConsole {
    pub fn new() -> Self {
        let output_wrapper = Arc::new(OutputWrapper::default());
        MESSAGE_BUS.add_handler(output_wrapper.clone()).and_log_if_err();
        Self { code: "".into(), eval_code: "".into(), output: output_wrapper }
    }
}

impl Render for PythonConsole {
    fn render(&mut self, ui: &mut Ui) -> anyhow::Result<()> {
        let Self { code, eval_code, output } = self;

        let output_text = &mut output.output.lock().map_err(|_| { anyhow!("") })?;
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                string,
                "py",
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::SidePanel::right("result_pane")
            .resizable(false)
            .exact_width(300.0)
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::bottom("eval_pane").frame(Frame::default().inner_margin(5.0)).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Clear").clicked() {
                            output_text.clear();
                        }
                    })
                });

                Frame::none().fill(Color32::BLACK)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink(false).show(ui, |ui| {
                            ui.label(
                                RichText::new(output_text.as_str()).code()
                            );
                        });
                    });
            });

        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::bottom("eval_input_pane").frame(Frame::default()
                    .inner_margin(5.0)
                    .outer_margin(5.0)
                )
                    .show_inside(ui, |ui| {
                        let response = egui::TextEdit::singleline(eval_code)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .lock_focus(false)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter)
                            .return_key(None)
                            .margin(Margin::symmetric(4.0, 8.0))
                            .show(ui)
                            .response;

                        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            MESSAGE_BUS.dispatch(Message::ExecutePython { code: eval_code.clone(), eval: true });
                            // eval_code.clear();
                        }
                    });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(code)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );

                    if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command) {
                        MESSAGE_BUS.dispatch(Message::ExecutePython { code: code.clone(), eval: false });
                    }
                });
            });
        Ok(())
    }
}

impl Panel for PythonConsole {
    fn get_name(&self) -> &str {
        "Python REPL"
    }
}