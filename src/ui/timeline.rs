use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::app::{AppState, DragTarget};
use crate::ui::theme;

const HEIGHT: f32 = 58.0;
const TRACK_THICKNESS: f32 = 8.0;
const SIDE_PADDING: f32 = 14.0;
const HANDLE_WIDTH: f32 = 10.0;
const HANDLE_GRAB: f32 = 13.0;

#[derive(Default, Debug)]
pub struct TimelineAction {
    pub seek_to: Option<f64>,
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState) -> TimelineAction {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HEIGHT),
        Sense::click_and_drag(),
    );

    if state.duration <= 0.0 {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No media loaded",
            FontId::proportional(14.0),
            theme::TEXT_DIM,
        );
        return TimelineAction::default();
    }

    let track = Rect::from_min_max(
        Pos2::new(rect.left() + SIDE_PADDING, rect.center().y - TRACK_THICKNESS / 2.0),
        Pos2::new(rect.right() - SIDE_PADDING, rect.center().y + TRACK_THICKNESS / 2.0),
    );

    let duration = state.duration;
    let x_of = |seconds: f64| -> f32 {
        track.left() + (seconds / duration).clamp(0.0, 1.0) as f32 * track.width()
    };
    let seconds_at = |x: f32| -> f64 {
        (((x - track.left()) / track.width()).clamp(0.0, 1.0) as f64) * duration
    };

    let mut action = TimelineAction::default();
    let start_x = x_of(state.selection.start());
    let end_x = x_of(state.selection.end());

    // Resolved from where the press began, not from when egui first reports a
    // drag — by then the pointer may already be far from the handle. A quick
    // click that starts and ends between two frames only surfaces as `clicked`.
    let active = response.is_pointer_button_down_on() || response.dragged();

    if active {
        if state.drag_target == DragTarget::None {
            let press_x = ui
                .input(|i| i.pointer.press_origin())
                .map(|origin| origin.x)
                .or_else(|| response.interact_pointer_pos().map(|p| p.x));
            if let Some(x) = press_x {
                state.drag_target = grabbed_at(x, start_x, end_x);
            }
        }

        if let Some(pointer) = response.interact_pointer_pos() {
            match state.drag_target {
                DragTarget::Start => state.selection.set_start(seconds_at(pointer.x)),
                DragTarget::End => state.selection.set_end(seconds_at(pointer.x)),
                DragTarget::Playhead => action.seek_to = Some(seconds_at(pointer.x)),
                DragTarget::None => {}
            }
        }
    } else {
        if response.clicked()
            && let Some(pointer) = response.interact_pointer_pos()
            && grabbed_at(pointer.x, start_x, end_x) == DragTarget::Playhead
        {
            action.seek_to = Some(seconds_at(pointer.x));
        }
        state.drag_target = DragTarget::None;
    }

    state.scrubbing = state.drag_target == DragTarget::Playhead;

    let painter = ui.painter();
    painter.rect_filled(track, TRACK_THICKNESS / 2.0, theme::TRACK);

    let selection = Rect::from_min_max(
        Pos2::new(x_of(state.selection.start()), track.top()),
        Pos2::new(x_of(state.selection.end()), track.bottom()),
    );
    painter.rect_filled(
        selection,
        TRACK_THICKNESS / 2.0,
        theme::ACCENT.gamma_multiply(0.55),
    );

    let playhead_x = x_of(state.position);
    painter.line_segment(
        [
            Pos2::new(playhead_x, track.top() - 12.0),
            Pos2::new(playhead_x, track.bottom() + 12.0),
        ],
        Stroke::new(2.0, Color32::WHITE),
    );

    for (seconds, target) in [
        (state.selection.start(), DragTarget::Start),
        (state.selection.end(), DragTarget::End),
    ] {
        let x = x_of(seconds);
        let handle = Rect::from_center_size(
            Pos2::new(x, track.center().y),
            Vec2::new(HANDLE_WIDTH, TRACK_THICKNESS + 16.0),
        );
        let fill = if state.drag_target == target {
            Color32::WHITE
        } else {
            theme::ACCENT
        };
        painter.rect_filled(handle, 3.0, fill);
    }

    painter.text(
        Pos2::new(rect.left() + SIDE_PADDING, rect.top() + 2.0),
        Align2::LEFT_TOP,
        format_time(state.selection.start()),
        FontId::monospace(11.0),
        theme::TEXT_DIM,
    );
    painter.text(
        Pos2::new(rect.right() - SIDE_PADDING, rect.top() + 2.0),
        Align2::RIGHT_TOP,
        format_time(state.selection.end()),
        FontId::monospace(11.0),
        theme::TEXT_DIM,
    );
    painter.text(
        rect.left_bottom() + Vec2::new(0.0, -2.0),
        Align2::LEFT_BOTTOM,
        format_time(state.position),
        FontId::monospace(11.0),
        theme::TEXT_DIM,
    );

    action
}

fn grabbed_at(x: f32, start_x: f32, end_x: f32) -> DragTarget {
    let to_start = (x - start_x).abs();
    let to_end = (x - end_x).abs();

    if to_start <= HANDLE_GRAB && to_start <= to_end {
        DragTarget::Start
    } else if to_end <= HANDLE_GRAB {
        DragTarget::End
    } else {
        DragTarget::Playhead
    }
}

pub fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

#[cfg(test)]
mod tests {
    use super::{DragTarget, format_time, grabbed_at};

    #[test]
    fn formats_durations_as_hours_minutes_seconds() {
        assert_eq!(format_time(0.0), "00:00:00");
        assert_eq!(format_time(59.0), "00:00:59");
        assert_eq!(format_time(60.0), "00:01:00");
        assert_eq!(format_time(3599.0), "00:59:59");
        assert_eq!(format_time(3600.0), "01:00:00");
        assert_eq!(format_time(3661.0), "01:01:01");
        assert_eq!(format_time(86_400.0), "24:00:00");
    }

    #[test]
    fn negative_and_fractional_values_are_safe() {
        assert_eq!(format_time(-5.0), "00:00:00");
        assert_eq!(format_time(1.4), "00:00:01");
        assert_eq!(format_time(1.6), "00:00:02");
    }

    #[test]
    fn pressing_near_a_handle_grabs_it() {
        assert_eq!(grabbed_at(100.0, 100.0, 300.0), DragTarget::Start);
        assert_eq!(grabbed_at(300.0, 100.0, 300.0), DragTarget::End);
        assert_eq!(grabbed_at(310.0, 100.0, 300.0), DragTarget::End);
        assert_eq!(grabbed_at(200.0, 100.0, 300.0), DragTarget::Playhead);
    }

    #[test]
    fn the_nearest_handle_wins_when_they_overlap() {
        assert_eq!(grabbed_at(100.0, 100.0, 104.0), DragTarget::Start);
        assert_eq!(grabbed_at(104.0, 100.0, 104.0), DragTarget::End);
    }
}
