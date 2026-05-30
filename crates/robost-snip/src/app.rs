use eframe::egui;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use image::{imageops, RgbaImage};
use robost_template::{Anchor, MaskRegion, Rect, TemplateMeta, WindowPoint};
use robost_vision::{ScreenPoint, TemplateMatcher};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem},
    TrayIcon, TrayIconBuilder,
};

// ===== Editing sub-types =====

#[derive(Clone)]
struct AnchorDef {
    px_x: i32,
    px_y: i32,
    label: Option<String>,
}

#[derive(Clone)]
struct MaskDef {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

#[derive(Clone, PartialEq)]
enum EditMode {
    View,
    AddAnchor,
    AddMask,
}

#[derive(Clone)]
enum LiveResult {
    Matched { score: f32, x: i32, y: i32 },
    NotFound,
}

// ===== Popup message shown as a floating egui Window =====

struct PopupMsg {
    title: &'static str,
    message: String,
    is_error: bool,
    /// If true, hide the viewport after the user dismisses this popup.
    then_hide: bool,
    /// Text to copy when the user presses the copy button (success only).
    copy_text: Option<String>,
}

// ===== Overlay state machine =====

enum OverlayState {
    Hidden,
    Selecting {
        texture: egui::TextureHandle,
        image: RgbaImage,
        drag_start: Option<egui::Pos2>,
    },
    Editing {
        screen_texture: egui::TextureHandle,
        screen_image: RgbaImage,
        template_texture: egui::TextureHandle,
        template_img: RgbaImage,
        anchors: Vec<AnchorDef>,
        masks: Vec<MaskDef>,
        /// Start position of in-progress mask drag, in preview display coords.
        mask_drag: Option<egui::Pos2>,
        multi_scale: bool,
        edit_mode: EditMode,
        label_input: String,
        live_result: Option<LiveResult>,
    },
}

// ===== Tray app =====

fn make_icon() -> tray_icon::Icon {
    const S: u32 = 16;
    let mut rgba = vec![0u8; (S * S * 4) as usize];
    for chunk in rgba.chunks_mut(4) {
        chunk[0] = 0;
        chunk[1] = 180;
        chunk[2] = 80;
        chunk[3] = 255;
    }
    tray_icon::Icon::from_rgba(rgba, S, S).expect("tray icon")
}

pub struct SnipApp {
    _tray: TrayIcon,
    quit_id: MenuId,
    capture_id: MenuId,
    help_id: MenuId,
    _hotkey_manager: GlobalHotKeyManager,
    capture_hotkey_id: u32,
    state: OverlayState,
    /// Popup shown to the user (error / success notification).
    popup_msg: Option<PopupMsg>,
    /// Whether to show the help window.
    show_help: bool,
    /// Inline error displayed inside the editing window (e.g. save failure).
    inline_error: Option<String>,
    /// When set, show "too small" hint until this instant, then hide overlay.
    too_small_until: Option<std::time::Instant>,
    /// When true, run live test this frame (window was hidden last frame for clean capture).
    pending_live_test: bool,
}

impl SnipApp {
    pub fn new() -> Self {
        let menu = Menu::new();
        let capture_item = MenuItem::new("キャプチャ開始 (Ctrl+Shift+C)", true, None);
        let capture_id = capture_item.id().clone();
        let help_item = MenuItem::new("使い方", true, None);
        let help_id = help_item.id().clone();
        let quit_item = MenuItem::new("robost-snip を終了", true, None);
        let quit_id = quit_item.id().clone();
        menu.append_items(&[&capture_item, &help_item, &quit_item])
            .expect("menu append");

        let tray = TrayIconBuilder::new()
            .with_icon(make_icon())
            .with_menu(Box::new(menu))
            .with_tooltip("robost-snip — Ctrl+Shift+C でキャプチャ")
            .build()
            .expect("tray build");

        let hotkey_manager = GlobalHotKeyManager::new().expect("hotkey manager");
        let capture_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyC);
        let capture_hotkey_id = capture_hotkey.id();
        hotkey_manager
            .register(capture_hotkey)
            .expect("register hotkey");

        tracing::info!("tray icon created; Ctrl+Shift+C registered");
        Self {
            _tray: tray,
            quit_id,
            capture_id,
            help_id,
            _hotkey_manager: hotkey_manager,
            capture_hotkey_id,
            state: OverlayState::Hidden,
            popup_msg: None,
            show_help: false,
            inline_error: None,
            too_small_until: None,
            pending_live_test: false,
        }
    }

    fn start_capture(&mut self, ctx: &egui::Context) {
        match robost_capture::capture_screen() {
            Ok(img) => {
                let size = [img.width() as usize, img.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
                let texture = ctx.load_texture(
                    "screen_capture",
                    color_image,
                    egui::TextureOptions::default(),
                );
                self.state = OverlayState::Selecting {
                    texture,
                    image: img,
                    drag_start: None,
                };
                self.inline_error = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
                tracing::info!("overlay shown");
            }
            Err(e) => {
                tracing::error!("capture failed: {e}");
                self.popup_msg = Some(PopupMsg {
                    title: "キャプチャエラー",
                    message: format!(
                        "画面のキャプチャに失敗しました。\n\
                         別のアプリを表示してから Ctrl+Shift+C を押してください。\n\n\
                         詳細: {e}"
                    ),
                    is_error: true,
                    then_hide: true,
                    copy_text: None,
                });
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            }
        }
    }

    fn hide(&mut self, ctx: &egui::Context) {
        self.state = OverlayState::Hidden;
        self.inline_error = None;
        self.too_small_until = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        tracing::info!("overlay hidden");
    }

    /// After rectangle selection, transition Selecting → Editing.
    fn enter_editing(&mut self, sel: egui::Rect, screen: egui::Rect, ctx: &egui::Context) {
        let (screen_texture, screen_image) =
            match std::mem::replace(&mut self.state, OverlayState::Hidden) {
                OverlayState::Selecting { texture, image, .. } => (texture, image),
                other => {
                    self.state = other;
                    return;
                }
            };

        let x = ((sel.min.x / screen.width()) * screen_image.width() as f32) as u32;
        let y = ((sel.min.y / screen.height()) * screen_image.height() as f32) as u32;
        let w = ((sel.width() / screen.width()) * screen_image.width() as f32).max(1.0) as u32;
        let h = ((sel.height() / screen.height()) * screen_image.height() as f32).max(1.0) as u32;
        let x = x.min(screen_image.width().saturating_sub(1));
        let y = y.min(screen_image.height().saturating_sub(1));
        let w = w.min(screen_image.width() - x).max(1);
        let h = h.min(screen_image.height() - y).max(1);
        let template_img = imageops::crop_imm(&screen_image, x, y, w, h).to_image();

        let size = [
            template_img.width() as usize,
            template_img.height() as usize,
        ];
        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, template_img.as_raw());
        let template_texture = ctx.load_texture(
            "template_preview",
            color_img,
            egui::TextureOptions::default(),
        );

        self.state = OverlayState::Editing {
            screen_texture,
            screen_image,
            template_texture,
            template_img,
            anchors: vec![],
            masks: vec![],
            mask_drag: None,
            multi_scale: true,
            edit_mode: EditMode::View,
            label_input: String::new(),
            live_result: None,
        };
        tracing::info!("entered editing mode");
    }

    fn draw_selecting(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide(ctx);
            return;
        }

        let texture_id = match &self.state {
            OverlayState::Selecting { texture, .. } => texture.id(),
            _ => return,
        };
        let screen_rect = ctx.viewport_rect();

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("select_bg"),
        ));
        painter.image(
            texture_id,
            screen_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        painter.rect_filled(
            screen_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100),
        );

        // Hint text at top
        let hint_painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("select_hint"),
        ));
        hint_painter.text(
            egui::pos2(screen_rect.center().x, 22.0),
            egui::Align2::CENTER_TOP,
            "ドラッグで範囲を選択（Escape でキャンセル）",
            egui::FontId::proportional(18.0),
            egui::Color32::WHITE,
        );
        // Show "too small" warning while timer is active
        if let Some(until) = self.too_small_until {
            if std::time::Instant::now() < until {
                hint_painter.text(
                    egui::pos2(screen_rect.center().x, 50.0),
                    egui::Align2::CENTER_TOP,
                    "選択範囲が小さすぎます。もう一度ドラッグしてください",
                    egui::FontId::proportional(16.0),
                    egui::Color32::from_rgb(255, 220, 80),
                );
            }
        }

        let pointer_pos = ctx.input(|i| i.pointer.latest_pos());
        let pressed = ctx.input(|i| i.pointer.primary_pressed());
        let released = ctx.input(|i| i.pointer.primary_released());

        if pressed {
            if let OverlayState::Selecting { drag_start, .. } = &mut self.state {
                *drag_start = pointer_pos;
            }
            // Cancel "too small" auto-hide timer when the user starts a new drag.
            self.too_small_until = None;
        }

        let drag_start = match &self.state {
            OverlayState::Selecting { drag_start, .. } => *drag_start,
            _ => return,
        };

        let sel_rect = drag_start
            .zip(pointer_pos)
            .map(|(s, c)| egui::Rect::from_two_pos(s, c));

        if let Some(r) = sel_rect {
            let uv = egui::Rect::from_min_max(
                egui::pos2(
                    r.min.x / screen_rect.width(),
                    r.min.y / screen_rect.height(),
                ),
                egui::pos2(
                    r.max.x / screen_rect.width(),
                    r.max.y / screen_rect.height(),
                ),
            );
            painter.image(texture_id, r, uv, egui::Color32::WHITE);
            painter.rect_stroke(
                r,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 180, 255)),
                egui::StrokeKind::Middle,
            );
        }

        if released {
            if let Some(r) = sel_rect {
                if r.width() > 4.0 && r.height() > 4.0 {
                    self.too_small_until = None;
                    self.enter_editing(r, screen_rect, ctx);
                    return;
                }
            }
            // Too small — start timer, reset drag so user can retry
            self.too_small_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(1500));
            if let OverlayState::Selecting {
                ref mut drag_start, ..
            } = self.state
            {
                *drag_start = None;
            }
            ctx.request_repaint();
        }
    }

    fn draw_editing(&mut self, ctx: &egui::Context) {
        let inline_error = self.inline_error.clone();
        let screen_rect = ctx.viewport_rect();

        // Draw frozen background + live test highlight
        {
            let (bg_id, tmpl_w, tmpl_h, img_w, img_h, live_clone) = match &self.state {
                OverlayState::Editing {
                    screen_texture,
                    screen_image,
                    template_img,
                    live_result,
                    ..
                } => (
                    screen_texture.id(),
                    template_img.width(),
                    template_img.height(),
                    screen_image.width(),
                    screen_image.height(),
                    live_result.clone(),
                ),
                _ => return,
            };

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Background,
                egui::Id::new("edit_bg"),
            ));
            painter.image(
                bg_id,
                screen_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            painter.rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            );

            if let Some(LiveResult::Matched { x, y, .. }) = live_clone {
                let highlight = egui::Rect::from_min_size(
                    egui::pos2(
                        screen_rect.min.x + x as f32 / img_w as f32 * screen_rect.width(),
                        screen_rect.min.y + y as f32 / img_h as f32 * screen_rect.height(),
                    ),
                    egui::vec2(
                        tmpl_w as f32 / img_w as f32 * screen_rect.width(),
                        tmpl_h as f32 / img_h as f32 * screen_rect.height(),
                    ),
                );
                painter.rect_stroke(
                    highlight,
                    0.0,
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 100)),
                    egui::StrokeKind::Outside,
                );
            }
        }

        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let mut do_cancel = escape;
        let mut do_save = false;
        let mut do_live_test = false;

        egui::Window::new("テンプレートエディター")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(true)
            .min_size(egui::vec2(460.0, 400.0))
            .default_size(egui::vec2(460.0, 580.0))
            .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let OverlayState::Editing {
                    template_texture,
                    template_img,
                    anchors,
                    masks,
                    mask_drag,
                    multi_scale,
                    edit_mode,
                    label_input,
                    live_result,
                    ..
                } = &mut self.state
                else {
                    return;
                };

                ui.label(format!(
                    "テンプレート: {} × {} px",
                    template_img.width(),
                    template_img.height()
                ));
                ui.separator();

                // --- Template preview ---
                let tw = template_img.width() as f32;
                let th = template_img.height() as f32;
                let scale = (420.0f32 / tw).min(280.0 / th).clamp(0.5, 8.0);
                let disp = egui::vec2(tw * scale, th * scale);

                let sense = match edit_mode {
                    EditMode::AddAnchor => egui::Sense::click(),
                    EditMode::AddMask => egui::Sense::click_and_drag(),
                    EditMode::View => egui::Sense::hover(),
                };
                let (preview_rect, response) = ui.allocate_exact_size(disp, sense);
                let pp = ui.painter_at(preview_rect);

                pp.image(
                    template_texture.id(),
                    preview_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Draw masks
                for m in masks.iter() {
                    let mr = egui::Rect::from_min_size(
                        egui::pos2(
                            preview_rect.min.x + m.x as f32 * scale,
                            preview_rect.min.y + m.y as f32 * scale,
                        ),
                        egui::vec2(m.w as f32 * scale, m.h as f32 * scale),
                    );
                    pp.rect_filled(mr, 0.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, 80));
                    pp.rect_stroke(
                        mr,
                        0.0,
                        egui::Stroke::new(1.5, egui::Color32::RED),
                        egui::StrokeKind::Middle,
                    );
                }

                // In-progress mask drag preview
                if *edit_mode == EditMode::AddMask {
                    if let (Some(ds), Some(cur)) = (*mask_drag, response.interact_pointer_pos()) {
                        let dr = egui::Rect::from_two_pos(ds, cur);
                        pp.rect_filled(dr, 0.0, egui::Color32::from_rgba_unmultiplied(255, 165, 0, 80));
                        pp.rect_stroke(
                            dr,
                            0.0,
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 165, 0)),
                            egui::StrokeKind::Middle,
                        );
                    }
                }

                // Draw anchors
                let cross_r = 7.0 * scale.min(2.0);
                for (i, a) in anchors.iter().enumerate() {
                    let ap = egui::pos2(
                        preview_rect.min.x + a.px_x as f32 * scale,
                        preview_rect.min.y + a.px_y as f32 * scale,
                    );
                    let c = egui::Color32::YELLOW;
                    pp.circle_filled(ap, 5.0 * scale.min(2.0), c);
                    pp.line_segment([ap - egui::vec2(cross_r, 0.0), ap + egui::vec2(cross_r, 0.0)], egui::Stroke::new(1.5, c));
                    pp.line_segment([ap - egui::vec2(0.0, cross_r), ap + egui::vec2(0.0, cross_r)], egui::Stroke::new(1.5, c));
                    pp.text(ap + egui::vec2(8.0, -8.0), egui::Align2::LEFT_TOP, (i + 1).to_string(), egui::FontId::monospace(11.0), c);
                }

                // Handle interactions
                match edit_mode {
                    EditMode::AddAnchor => {
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let px = ((pos.x - preview_rect.min.x) / scale) as i32;
                                let py = ((pos.y - preview_rect.min.y) / scale) as i32;
                                let label = if label_input.is_empty() {
                                    None
                                } else {
                                    Some(label_input.clone())
                                };
                                anchors.push(AnchorDef { px_x: px, px_y: py, label });
                            }
                        }
                    }
                    EditMode::AddMask => {
                        if response.drag_started() {
                            *mask_drag = response.interact_pointer_pos();
                        }
                        if response.drag_stopped() {
                            let end = response.interact_pointer_pos();
                            if let (Some(start), Some(end)) = (*mask_drag, end) {
                                let r = egui::Rect::from_two_pos(start, end);
                                let px = ((r.min.x - preview_rect.min.x) / scale) as i32;
                                let py = ((r.min.y - preview_rect.min.y) / scale) as i32;
                                let pw = (r.width() / scale) as u32;
                                let ph = (r.height() / scale) as u32;
                                if pw > 1 && ph > 1 {
                                    masks.push(MaskDef { x: px, y: py, w: pw, h: ph });
                                }
                            }
                            *mask_drag = None;
                        }
                    }
                    EditMode::View => {}
                }

                ui.separator();

                // --- Mode selector ---
                ui.horizontal(|ui| {
                    ui.label("モード:");
                    if ui.selectable_label(*edit_mode == EditMode::View, "表示").clicked() {
                        *edit_mode = EditMode::View;
                    }
                    if ui
                        .selectable_label(*edit_mode == EditMode::AddAnchor, "+ アンカー")
                        .on_hover_text(
                            "クリック基準点を設定します。\nテンプレートのどこをクリックするかを指定できます。",
                        )
                        .clicked()
                    {
                        *edit_mode = EditMode::AddAnchor;
                    }
                    if ui
                        .selectable_label(*edit_mode == EditMode::AddMask, "+ マスク")
                        .on_hover_text(
                            "マッチング除外領域を設定します。\nタイムスタンプなど変動する部分を除外します。",
                        )
                        .clicked()
                    {
                        *edit_mode = EditMode::AddMask;
                    }
                });

                if *edit_mode == EditMode::AddAnchor {
                    ui.horizontal(|ui| {
                        ui.label("ラベル:");
                        ui.text_edit_singleline(label_input);
                        ui.label("（テンプレートをクリックして配置）");
                    });
                }
                if *edit_mode == EditMode::AddMask {
                    ui.label(
                        egui::RichText::new("ドラッグで除外領域を描画してください")
                            .color(egui::Color32::from_rgb(255, 165, 0))
                            .small(),
                    );
                }

                // --- Anchors list ---
                if !anchors.is_empty() {
                    let n = anchors.len();
                    egui::CollapsingHeader::new(format!("アンカー ({n})"))
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut rm = None;
                            for (i, a) in anchors.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let lbl = a.label.as_deref().unwrap_or("—");
                                    ui.label(format!("#{} ({}, {})  \"{}\"", i + 1, a.px_x, a.px_y, lbl));
                                    if ui.small_button("✕").clicked() {
                                        rm = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = rm {
                                anchors.remove(i);
                            }
                        });
                }

                // --- Masks list ---
                if !masks.is_empty() {
                    let n = masks.len();
                    egui::CollapsingHeader::new(format!("マスク ({n})"))
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut rm = None;
                            for (i, m) in masks.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("#{} ({}, {}, {}×{})", i + 1, m.x, m.y, m.w, m.h));
                                    if ui.small_button("✕").clicked() {
                                        rm = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = rm {
                                masks.remove(i);
                            }
                        });
                }

                ui.separator();

                ui.checkbox(multi_scale, "マルチスケール生成（125% / 150%） — DPI 変動対応");

                ui.separator();

                // --- Live test ---
                ui.horizontal(|ui| {
                    if ui
                        .button("▶ マッチング確認")
                        .on_hover_text("現在の画面をキャプチャして、採取した画像が存在するかテストします")
                        .clicked()
                    {
                        do_live_test = true;
                    }
                    match live_result {
                        Some(LiveResult::Matched { score, x, y }) => {
                            ui.colored_label(
                                egui::Color32::from_rgb(0, 200, 80),
                                format!("✓ 見つかりました（一致度 {:.0}%）　位置 ({x}, {y})", *score * 100.0),
                            );
                        }
                        Some(LiveResult::NotFound) => {
                            ui.colored_label(egui::Color32::RED, "✗ 見つかりませんでした");
                        }
                        None => {}
                    }
                });

                // Inline error (e.g. save failure)
                if let Some(ref err) = inline_error {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(220, 60, 60), err);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 保存").clicked() {
                        do_save = true;
                    }
                    if ui.button("✕ キャンセル").clicked() {
                        do_cancel = true;
                    }
                });
            }); // ScrollArea
            }); // Window

        if do_cancel {
            self.hide(ctx);
        } else if do_save {
            self.save_template(ctx);
        } else if do_live_test {
            // Hide the overlay window so the capture sees the underlying application.
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.pending_live_test = true;
        }
    }

    fn run_live_test(&mut self) {
        // Capture a fresh live screenshot so the test reflects actual screen state.
        let live_screen = match robost_capture::capture_screen() {
            Ok(img) => img,
            Err(e) => {
                tracing::warn!("live test capture failed, falling back to frozen: {e}");
                // Fall back to frozen screen image
                match &self.state {
                    OverlayState::Editing { screen_image, .. } => screen_image.clone(),
                    _ => return,
                }
            }
        };

        let (screen_img, tmpl_img, mask_regions) = match &self.state {
            OverlayState::Editing {
                template_img,
                masks,
                ..
            } => {
                let mask_regions: Vec<MaskRegion> = masks
                    .iter()
                    .map(|m| MaskRegion {
                        rect: Rect {
                            x: m.x,
                            y: m.y,
                            width: m.w,
                            height: m.h,
                        },
                        label: None,
                    })
                    .collect();
                (live_screen, template_img.clone(), mask_regions)
            }
            _ => return,
        };

        let result = TemplateMatcher::default().find_with_masks(
            &screen_img,
            &tmpl_img,
            ScreenPoint { x: 0, y: 0 },
            &mask_regions,
        );

        let live = match result {
            Ok(m) => LiveResult::Matched {
                score: m.score,
                x: m.location.x,
                y: m.location.y,
            },
            Err(_) => LiveResult::NotFound,
        };

        if let OverlayState::Editing { live_result, .. } = &mut self.state {
            *live_result = Some(live);
        }
    }

    /// Save the template while keeping the editing state alive on failure.
    fn save_template(&mut self, ctx: &egui::Context) {
        let (tmpl_img, anchors_c, masks_c, multi_scale) = match &self.state {
            OverlayState::Editing {
                template_img,
                anchors,
                masks,
                multi_scale,
                ..
            } => (
                template_img.clone(),
                anchors.clone(),
                masks.clone(),
                *multi_scale,
            ),
            _ => return,
        };

        let ppp = ctx.pixels_per_point();
        match do_save(&tmpl_img, &anchors_c, &masks_c, multi_scale, ppp) {
            Ok((png_path, scale_warnings)) => {
                tracing::info!(path = %png_path.display(), "template saved");
                self.inline_error = None;
                self.state = OverlayState::Hidden;
                // Keep viewport visible for success popup; hide after OK.
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                let path_str = png_path.to_string_lossy().into_owned();
                let mut message = format!("テンプレートを保存しました:\n{path_str}");
                if !scale_warnings.is_empty() {
                    message.push_str("\n\n⚠ マルチスケール保存の警告:");
                    for w in &scale_warnings {
                        message.push_str(&format!("\n  • {w}"));
                    }
                }
                self.popup_msg = Some(PopupMsg {
                    title: "保存完了",
                    message,
                    is_error: false,
                    then_hide: true,
                    copy_text: Some(path_str),
                });
            }
            Err(msg) => {
                tracing::error!("save failed: {msg}");
                self.inline_error = Some(format!("保存に失敗しました: {msg}"));
                // Editing state stays open so the user can retry or cancel.
            }
        }
    }
}

/// Save template image + YAML metadata to the `templates/` directory.
/// Returns `(main_png_path, scale_warnings)`.
fn do_save(
    tmpl_img: &RgbaImage,
    anchors: &[AnchorDef],
    masks: &[MaskDef],
    multi_scale: bool,
    pixels_per_point: f32,
) -> Result<(std::path::PathBuf, Vec<String>), String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let stem = format!("template_{ts}");
    let dir = dirs::document_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("robost")
        .join("templates");

    std::fs::create_dir_all(&dir).map_err(|e| format!("フォルダ作成: {e}"))?;

    let png_name = format!("{stem}.png");
    let png_path = dir.join(&png_name);
    tmpl_img
        .save(&png_path)
        .map_err(|e| format!("PNG保存: {e}"))?;

    let mut scale_warnings: Vec<String> = Vec::new();
    if multi_scale {
        for (pct, factor) in [(125u32, 1.25f32), (150, 1.5)] {
            let nw = ((tmpl_img.width() as f32) * factor) as u32;
            let nh = ((tmpl_img.height() as f32) * factor) as u32;
            let scaled = imageops::resize(tmpl_img, nw, nh, imageops::FilterType::Lanczos3);
            let path = dir.join(format!("{stem}_{pct}pct.png"));
            if let Err(e) = scaled.save(&path) {
                let w = format!("{pct}% バリアント保存失敗: {e}");
                tracing::warn!("{w}");
                scale_warnings.push(w);
            }
        }
    }

    let meta = TemplateMeta {
        label: stem.clone(),
        image_path: png_name,
        anchors: anchors
            .iter()
            .map(|a| Anchor {
                offset: WindowPoint {
                    x: a.px_x,
                    y: a.px_y,
                },
                label: a.label.clone(),
            })
            .collect(),
        masks: masks
            .iter()
            .map(|m| MaskRegion {
                rect: Rect {
                    x: m.x,
                    y: m.y,
                    width: m.w,
                    height: m.h,
                },
                label: None,
            })
            .collect(),
        dpi_scale: pixels_per_point,
    };

    let yaml_path = dir.join(format!("{stem}.yaml"));
    let yaml = serde_yml::to_string(&meta).map_err(|e| format!("YAML変換: {e}"))?;
    std::fs::write(&yaml_path, yaml).map_err(|e| format!("YAML保存: {e}"))?;

    Ok((png_path, scale_warnings))
}

impl eframe::App for SnipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Pending live test: window was hidden last frame; run capture now and restore.
        if self.pending_live_test {
            self.pending_live_test = false;
            self.run_live_test();
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.request_repaint();
            return;
        }

        // Handle tray menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_id {
                tracing::info!("quit requested via tray menu");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if event.id == self.capture_id {
                if matches!(self.state, OverlayState::Hidden) && self.popup_msg.is_none() {
                    self.start_capture(ctx);
                }
            } else if event.id == self.help_id {
                self.show_help = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            }
        }

        // Global hotkey — only active in Hidden state with no popup
        if matches!(self.state, OverlayState::Hidden) && self.popup_msg.is_none() && !self.show_help
        {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.id == self.capture_hotkey_id && event.state == HotKeyState::Pressed {
                    tracing::info!("Ctrl+Shift+C: starting capture");
                    self.start_capture(ctx);
                }
            }
        }

        // Show help window
        if self.show_help {
            let mut close_help = false;
            egui::Window::new("robost-snip 使い方")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(true)
                .default_size([420.0, 360.0])
                .min_size([320.0, 240.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.strong("【基本操作】");
                        ui.add_space(4.0);
                        ui.label("① 採取したい画面（ボタン、ドロップダウンなど）を表示する");
                        ui.label("② Ctrl+Shift+C を押すと画面が凍結する");
                        ui.label("③ テンプレートにしたい範囲をドラッグで選択する");
                        ui.label("④ アンカー・マスクを必要に応じて設定して「保存」する");
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.strong("【用語説明】");
                        ui.add_space(4.0);
                        ui.label("アンカー: クリック基準点（テンプレートのどこをクリックするか）");
                        ui.label("マスク: マッチング除外領域（タイムスタンプ等の変動箇所を除外）");
                        ui.label("マッチング確認: 採取画像が現在の凍結画面に存在するかテスト");
                        ui.add_space(12.0);
                        ui.separator();
                        if ui.button("閉じる").clicked()
                            || ctx.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            close_help = true;
                        }
                    });
                });
            if close_help {
                self.show_help = false;
                if matches!(self.state, OverlayState::Hidden) && self.popup_msg.is_none() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
            }
        }

        // Show popup message (success / error notification)
        if let Some(ref msg) = self.popup_msg {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            let title = msg.title;
            let is_error = msg.is_error;
            let message = msg.message.clone();
            let then_hide = msg.then_hide;
            let copy_text = msg.copy_text.clone();
            let mut dismiss = false;

            egui::Modal::new(egui::Id::new("snip_popup")).show(ctx, |ui| {
                ui.set_min_width(280.0);
                ui.strong(title);
                ui.add_space(4.0);
                let color = if is_error {
                    egui::Color32::from_rgb(220, 60, 60)
                } else {
                    egui::Color32::from_rgb(0, 180, 80)
                };
                ui.colored_label(color, &message);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked()
                        || ctx.input(|i| i.key_pressed(egui::Key::Enter))
                        || ctx.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        dismiss = true;
                    }
                    if let Some(ref copy) = copy_text {
                        if ui.button("📋 パスをコピー").clicked() {
                            ui.ctx().copy_text(copy.clone());
                        }
                    }
                });
            });

            if dismiss {
                self.popup_msg = None;
                if then_hide && matches!(self.state, OverlayState::Hidden) && !self.show_help {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
            }
        }

        // Auto-hide after "too small" hint timer expires
        if let Some(until) = self.too_small_until {
            if std::time::Instant::now() >= until {
                self.too_small_until = None;
                if matches!(self.state, OverlayState::Selecting { .. }) {
                    self.hide(ctx);
                }
            }
        }

        // Render overlay state
        if matches!(self.state, OverlayState::Selecting { .. }) {
            self.draw_selecting(ctx);
        } else if matches!(self.state, OverlayState::Editing { .. }) {
            self.draw_editing(ctx);
        }

        let interval = if matches!(self.state, OverlayState::Hidden)
            && self.popup_msg.is_none()
            && !self.show_help
        {
            std::time::Duration::from_millis(100)
        } else {
            std::time::Duration::from_millis(16)
        };
        ctx.request_repaint_after(interval);
    }
}
