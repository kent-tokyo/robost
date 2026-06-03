mod app;

// ── Design tokens (mirrors robost-editor/src/tokens.rs) ──────────────────────
const ACCENT: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(0x00, 0x78, 0xD4);
const WARNING: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(0xC1, 0x9C, 0x00);
const ROUNDING_SM: eframe::egui::CornerRadius = eframe::egui::CornerRadius::same(4);
const ROUNDING_MD: eframe::egui::CornerRadius = eframe::egui::CornerRadius::same(8);
const SPACING_XS: f32 = 4.0;
const SPACING_SM: f32 = 8.0;

fn setup_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    let candidates: &[&str] = &[
        // macOS
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        // Windows
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
        "C:\\Windows\\Fonts\\YuGothR.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJKjp-Regular.otf",
    ];

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "cjk".to_owned(),
                eframe::egui::FontData::from_owned(data).into(),
            );
            fonts
                .families
                .entry(eframe::egui::FontFamily::Proportional)
                .or_default()
                .push("cjk".to_owned());
            fonts
                .families
                .entry(eframe::egui::FontFamily::Monospace)
                .or_default()
                .push("cjk".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn apply_style(ctx: &eframe::egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.selection.bg_fill = ACCENT.gamma_multiply(0.45);
    style.visuals.window_corner_radius = ROUNDING_MD;
    style.visuals.menu_corner_radius = ROUNDING_MD;
    for ws in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
        &mut style.visuals.widgets.noninteractive,
    ] {
        ws.corner_radius = ROUNDING_SM;
    }
    style.spacing.item_spacing = eframe::egui::vec2(SPACING_SM, SPACING_XS);
    style.spacing.button_padding = eframe::egui::vec2(SPACING_SM, SPACING_XS);

    ctx.set_style(style);
}

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
            apply_style(&cc.egui_ctx);
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(app::SnipApp::new()))
        }),
    )
    .expect("eframe run");
}
