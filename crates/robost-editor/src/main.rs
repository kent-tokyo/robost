use anyhow::Result;
use eframe::egui;

mod ai_integration;
mod app;
mod canvas;
mod flow_helpers;
mod flowchart;
mod i18n;
mod property_panel;
mod settings;
mod state;
mod step_templates;
mod tokens;
mod types;

use state::EditorApp;

// ---- main -----------------------------------------------------------------

pub(crate) fn apply_style(ctx: &egui::Context, theme: &settings::Theme) {
    let mut style = (*ctx.global_style()).clone();

    // Base visuals — light or dark — applied first so overlays below take effect on top.
    style.visuals = match theme {
        settings::Theme::Light => egui::Visuals::light(),
        settings::Theme::Dark => egui::Visuals::dark(),
    };

    // ACCENT blue selection highlight (DESIGN.md §1.1)
    style.visuals.selection.bg_fill = tokens::ACCENT.gamma_multiply(0.45);

    // Consistent corner radii (DESIGN.md §3 / Appendix B)
    style.visuals.window_corner_radius = tokens::ROUNDING_MD;
    style.visuals.menu_corner_radius = tokens::ROUNDING_MD;
    for ws in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
        &mut style.visuals.widgets.noninteractive,
    ] {
        ws.corner_radius = tokens::ROUNDING_SM;
    }

    // Interactive states (DESIGN.md §9.2)
    // hover: bg_fill is already brighter than inactive in egui defaults;
    //   add an explicit bg_stroke to make focus rings visible.
    style.visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0, tokens::ACCENT.gamma_multiply(0.6));
    // active/pressed: inset shadow effect via a stronger border
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, tokens::ACCENT);
    // disabled (noninteractive): 0.38 opacity per §9.2
    // egui uses fg_stroke for text; reduce its alpha to signal disabled state.
    let nonint_fg = style.visuals.widgets.noninteractive.fg_stroke.color;
    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, nonint_fg.gamma_multiply(0.38));

    // Base spacing (DESIGN.md §3.1)
    style.spacing.item_spacing = egui::vec2(tokens::SPACING_SM, tokens::SPACING_XS);
    style.spacing.button_padding = egui::vec2(tokens::SPACING_SM, tokens::SPACING_XS);

    ctx.set_global_style(style);
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Phosphor icon font (embedded in egui-phosphor crate)
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

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
            fonts
                .font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("cjk".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let initial_path: Option<std::path::PathBuf> = std::env::args().nth(1).map(Into::into);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RPA シナリオエディター")
            .with_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "robost-editor",
        native_options,
        Box::new(move |cc| {
            // Build the app first so we can read the persisted theme before styling.
            let mut app = EditorApp::default();
            apply_style(&cc.egui_ctx, &app.settings.theme);
            setup_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            if let Some(p) = initial_path {
                app.load_file_path(&p);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}
