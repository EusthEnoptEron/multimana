use std::thread::sleep;
use std::time::Duration;
use eframe::{egui};
use winit::platform::windows::EventLoopBuilderExtWindows;

mod app;
mod python_console;

pub fn open_gui() {
    std::thread::spawn(|| {
        sleep(Duration::from_secs(5));
        
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0]),
            event_loop_builder: Some(Box::new(|it| {
                it.with_any_thread(true);
            })),
            ..Default::default()
        };
        eframe::run_native(
            "eframe template",
            native_options,
            Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
        ).expect("Failure running GUI");
    });
}


trait Render {
    fn render(&mut self, ui: &mut egui::Ui) -> anyhow::Result<()>;
}

trait Show : Render {
    fn show(&mut self, ctx: &egui::Context);
}

trait Panel : Render {
    fn get_name(&self) -> &str;
}