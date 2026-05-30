mod app;

fn main() {
    tracing_subscriber::fmt::init();
    robost_capture::init_dpi();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_visible(false)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "robost-snip",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Ok(Box::new(app::SnipApp::new()))
        }),
    )
    .expect("eframe run");
}
