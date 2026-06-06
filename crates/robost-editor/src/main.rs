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
    let dark = matches!(theme, settings::Theme::Dark);
    let mut style = (*ctx.global_style()).clone();

    // ── Base visuals ──────────────────────────────────────────────────────────
    style.visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    // ── VS Code–derived color overrides ───────────────────────────────────────
    // Override the default egui colors with the VS Code Dark+ palette.
    // All panel backgrounds are set explicitly in each panel's show() call;
    // here we only configure the global widget / window defaults.

    // Window and popup background = sidebar bg (most floating surfaces sit in sidebar context).
    let window_bg = if dark { tokens::SIDEBAR_BG } else { tokens::SIDEBAR_BG_LIGHT };
    style.visuals.window_fill = window_bg;
    style.visuals.panel_fill = window_bg;

    // Selection highlight (light mode: subtle blue background)
    let selection_bg = if dark { tokens::LIST_SELECTION } else { egui::Color32::from_rgb(0xE8, 0xF1, 0xF8) };
    style.visuals.selection.bg_fill = selection_bg;
    style.visuals.selection.stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);

    // ── Widget interaction states — background fills ONLY, no borders ─────────
    // VS Code principle: hover = subtle bg fill; focus = outline; never both.
    let hover_bg = if dark { tokens::LIST_HOVER } else { egui::Color32::from_rgb(0xF0, 0xF0, 0xF0) };
    let active_bg = if dark { tokens::LIST_SELECTION } else { egui::Color32::from_rgb(0xE0, 0xE6, 0xF6) };

    style.visuals.widgets.hovered.weak_bg_fill = hover_bg;
    style.visuals.widgets.hovered.bg_fill = hover_bg;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE; // ← no border on hover
    style.visuals.widgets.active.weak_bg_fill = active_bg;
    style.visuals.widgets.active.bg_fill = active_bg;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

    // Text colors
    let fg = if dark { tokens::FG_DEFAULT } else { egui::Color32::from_gray(38) };
    let fg_dim = if dark { tokens::FG_DIM } else { egui::Color32::from_gray(120) };
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, fg);
    style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg_dim);

    // Separators — subtle light gray dividers
    let sep = if dark { egui::Color32::from_gray(50) } else { egui::Color32::from_gray(220) };
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, sep);

    // ── Corner radii (VS Code: 0-2 px UI, 4 px cards) ────────────────────────
    style.visuals.window_corner_radius = tokens::ROUNDING_CARD;
    style.visuals.menu_corner_radius = tokens::ROUNDING_CARD;
    for ws in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
        &mut style.visuals.widgets.noninteractive,
    ] {
        ws.corner_radius = tokens::ROUNDING_UI;
    }

    // ── Spacing ───────────────────────────────────────────────────────────────
    // Slightly more padding than before to prevent labels touching panel edges.
    style.spacing.item_spacing = egui::vec2(tokens::SPACING_SM, 5.0);
    style.spacing.button_padding = egui::vec2(tokens::SPACING_SM, 4.0);
    style.spacing.window_margin = egui::Margin::same(8);
    style.spacing.menu_margin = egui::Margin::same(4);

    ctx.set_global_style(style);
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Phosphor icon font (embedded in egui-phosphor crate)
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    // CJK font candidates — first match wins.
    // macOS: prefer Hiragino Sans W3 (crisp Japanese) then fall back to Sans GB.
    // Windows: Meiryo (good hinting), Yu Gothic, MS Gothic.
    // Linux: Noto Sans CJK.
    let cjk_candidates: &[&str] = &[
        // macOS — W4 (regular weight) for readability in both light and dark modes.
        // W3 (light) is too thin at UI sizes (14 px body, 11 px small).
        "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        // Windows
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\YuGothR.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJKjp-Regular.otf",
    ];
    for path in cjk_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
            // Append CJK AFTER Phosphor icons in the priority list.
            // Order: [Ubuntu-Light, phosphor, cjk]
            // — Ubuntu-Light handles Latin; Phosphor handles icon codepoints;
            //   CJK is fallback for Japanese/Chinese glyphs neither can serve.
            // Inserting CJK at position 0 would let its .notdef glyph (□)
            // intercept Phosphor's PUA codepoints before Phosphor is reached.
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

    // Font sizes aligned with VS Code's 13 px baseline (DESIGN.md §Typography).
    use egui::{FontId, TextStyle};
    let mut style = (*ctx.global_style()).clone();
    style.text_styles = [
        (TextStyle::Small, FontId::proportional(11.0)),
        (TextStyle::Body, FontId::proportional(13.0)),
        (TextStyle::Button, FontId::proportional(13.0)),
        (TextStyle::Heading, FontId::proportional(15.0)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
    ]
    .into();
    ctx.set_global_style(style);
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
