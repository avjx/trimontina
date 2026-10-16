use eframe::egui::{self, Align2, Color32, CursorIcon, FontId, Pos2, Rect, Sense};

use crate::player::Player;
use crate::ui::theme;

pub fn show(ui: &mut egui::Ui, player: &Player, has_media: bool) -> bool {
    let area = ui.available_rect_before_wrap();
    let response = ui.interact(area, ui.id().with("preview-surface"), Sense::click());

    let Some(surface) = player.surface() else {
        return false;
    };

    if !has_media {
        ui.painter().text(
            area.center(),
            Align2::CENTER_CENTER,
            "No video loaded",
            FontId::proportional(15.0),
            theme::TEXT_DIM,
        );
        return false;
    }

    let available = area.size();
    let aspect = surface.aspect_ratio();
    let mut size = egui::vec2(available.x, available.x / aspect);
    if size.y > available.y {
        size = egui::vec2(available.y * aspect, available.y);
    }

    ui.painter().image(
        surface.texture_id(),
        Rect::from_center_size(area.center(), size),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    response.on_hover_cursor(CursorIcon::PointingHand).clicked()
}
