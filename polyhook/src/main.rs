use egui::Vec2;

#[derive(Default)]
struct App {}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(150.0)
                .show_inside(ui, |ui| {
                    ui.heading(format!(
                        "Hello, world! The window size is {} by {}",
                        ctx.screen_rect().width(),
                        ctx.screen_rect().height()
                    ));
                });

            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
                    ui.heading(format!(
                        "Hello, world! The window size is {} by {}",
                        ctx.screen_rect().width(),
                        ctx.screen_rect().height()
                    ));
                })
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(Vec2::new(1024.0, 768.0))
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Polyhook",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );

    Ok(())
}
