use std::sync::Arc;

use eframe::egui::{self, Color32, Galley, Pos2, Rect, Sense, Vec2};

use crate::ui::theme;

const FONT_SIZE: f32 = 18.0;
const REST_OFFSET: f32 = 1.4;
const HOVER_OFFSET: f32 = 4.4;
const CUT_TOP: f32 = 0.20;
const CUT_BOTTOM: f32 = 0.80;
const CUT_GAP: f32 = 0.13;

pub fn glitch_title(ui: &mut egui::Ui, text: &str, exporting: bool) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::new(FONT_SIZE, egui::FontFamily::Name(theme::WORDMARK.into())),
        Color32::PLACEHOLDER,
    );
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());

    let ctx = ui.ctx();
    let hover = ctx.animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.18);
    let offset = REST_OFFSET + (HOVER_OFFSET - REST_OFFSET) * hover;

    let sweep = exporting.then(|| (ctx.input(|i| i.time) as f32 * 0.9).fract());

    let painter = ui.painter();
    draw_split(painter, &galley, rect, offset);
    if let Some(phase) = sweep {
        draw_sweep(painter, &galley, rect, phase);
    }

    if exporting || (hover > 0.001 && hover < 0.999) {
        ctx.request_repaint();
    }
    response
}

fn cut_at(height_fraction: f32, shift: f32) -> f32 {
    ((CUT_BOTTOM - height_fraction - shift) / (CUT_BOTTOM - CUT_TOP)).clamp(0.0, 1.0)
}

fn rows(rect: Rect) -> impl Iterator<Item = (f32, f32, f32)> {
    let slices = rect.height().ceil().max(4.0) as usize;
    let step = rect.height() / slices as f32;
    (0..slices).map(move |i| {
        let y0 = rect.top() + i as f32 * step;
        (y0, y0 + step, (i as f32 + 0.5) / slices as f32)
    })
}

fn draw_split(painter: &egui::Painter, galley: &Arc<Galley>, rect: Rect, offset: f32) {
    let tint = theme::ACCENT_SOFT;

    let half_gap = CUT_GAP * 0.5;

    for (y0, y1, fraction) in rows(rect) {
        let upper_edge = rect.left() + cut_at(fraction, half_gap) * rect.width();
        let lower_edge = rect.left() + cut_at(fraction, -half_gap) * rect.width();

        let upper = Rect::from_min_max(
            Pos2::new(rect.left() - offset, y0),
            Pos2::new(upper_edge, y1),
        );
        painter.with_clip_rect(upper).galley(
            rect.min + Vec2::new(-offset, -offset),
            galley.clone(),
            tint,
        );

        let lower = Rect::from_min_max(
            Pos2::new(lower_edge, y0),
            Pos2::new(rect.right() + offset, y1),
        );
        painter.with_clip_rect(lower).galley(
            rect.min + Vec2::new(offset, offset),
            galley.clone(),
            tint,
        );
    }
}

fn draw_sweep(painter: &egui::Painter, galley: &Arc<Galley>, rect: Rect, phase: f32) {
    const BAND: f32 = 0.10;
    const TILT: f32 = 0.22;
    let travel = phase * 1.2 - 0.1;
    let tint = theme::ACCENT_BRIGHT;

    for (y0, y1, fraction) in rows(rect) {
        let centre = travel + (fraction - 0.5) * TILT;
        let band = Rect::from_min_max(
            Pos2::new(rect.left() + (centre - BAND) * rect.width(), y0),
            Pos2::new(rect.left() + (centre + BAND) * rect.width(), y1),
        );
        painter
            .with_clip_rect(band)
            .galley(rect.min, galley.clone(), tint);
    }
}
