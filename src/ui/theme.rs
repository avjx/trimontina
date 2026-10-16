use eframe::egui::{self, Color32, CornerRadius, Margin, Shadow, Stroke};

pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0xE6, 0x76);
pub const BACKGROUND: Color32 = Color32::from_rgb(0x08, 0x08, 0x0C);
pub const TRACK: Color32 = Color32::from_rgb(0x38, 0x38, 0x44);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0xA0, 0xA0, 0xAE);
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x5C, 0x5C);
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x00, 0xE6, 0x76, 210);
pub const ACCENT_BRIGHT: Color32 = Color32::from_rgba_unmultiplied_const(0xEC, 0xFF, 0xF4, 230);

const CARD_RADIUS: u8 = 16;

const WORDMARK_BYTES: &[u8] = include_bytes!("../../assets/fonts/Anton-Regular.ttf");
pub const WORDMARK: &str = "wordmark";

fn alpha(value: u8, opacity: f32) -> u8 {
    (value as f32 * opacity.clamp(0.0, 1.0)).round() as u8
}

pub fn card(opacity: f32) -> egui::Frame {
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(18, 18, 24, alpha(110, opacity)))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, alpha(26, opacity)),
        ))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 12))
        .shadow(Shadow {
            offset: [0, 0],
            blur: 24,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(0, 0, 0, alpha(120, opacity)),
        })
}

pub fn chip(fill: Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(fill)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 8))
}

pub fn primary_button(label: &str) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(label.to_owned())
            .color(BACKGROUND)
            .strong(),
    )
    .fill(ACCENT)
    .corner_radius(10.0)
}

pub fn apply(ctx: &egui::Context) {
    install_wordmark(ctx);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = Color32::from_rgb(0x14, 0x14, 0x1A);
    visuals.extreme_bg_color = TRACK;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    let translucent = |a: u8| Color32::from_rgba_unmultiplied(255, 255, 255, a);
    visuals.widgets.inactive.bg_fill = translucent(16);
    visuals.widgets.inactive.weak_bg_fill = translucent(16);
    visuals.widgets.hovered.bg_fill = translucent(34);
    visuals.widgets.hovered.weak_bg_fill = translucent(34);
    visuals.widgets.active.bg_fill = ACCENT.gamma_multiply(0.75);
    visuals.widgets.active.weak_bg_fill = ACCENT.gamma_multiply(0.75);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, translucent(20));

    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(10);
    }

    ctx.set_visuals(visuals);

    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    ctx.set_global_style(style);
}

fn install_wordmark(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        WORDMARK.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(WORDMARK_BYTES)),
    );
    fonts.families.insert(
        egui::FontFamily::Name(WORDMARK.into()),
        vec![WORDMARK.to_owned()],
    );
    ctx.set_fonts(fonts);
}
