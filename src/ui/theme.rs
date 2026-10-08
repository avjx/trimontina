use eframe::egui::{self, Color32, Stroke};

pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0xE6, 0x76);
pub const BACKGROUND: Color32 = Color32::from_rgb(0x12, 0x12, 0x16);
pub const SURFACE: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x22);
pub const TRACK: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x38);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9A, 0x9A, 0xA6);
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x5C, 0x5C);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = TRACK;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.widgets.inactive.bg_fill = TRACK;
    visuals.widgets.inactive.weak_bg_fill = TRACK;
    visuals.widgets.hovered.bg_fill = ACCENT.gamma_multiply(0.6);
    visuals.widgets.hovered.weak_bg_fill = ACCENT.gamma_multiply(0.6);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.weak_bg_fill = ACCENT;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    ctx.set_global_style(style);
}
