use eframe::egui;

use crate::app::{AppState, ExportStatus};
use crate::ui::{theme, timeline};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlAction {
    OpenMedia,
    TogglePlay,
    Export,
    SetVolume(f32),
    ToggleMute,
}

pub fn show(ui: &mut egui::Ui, state: &AppState) -> Option<ControlAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        let (glyph, hint) = if state.playing {
            ("⏸", "Pause")
        } else {
            ("▶", "Play")
        };
        let play = egui::Button::new(egui::RichText::new(glyph).size(17.0).color(theme::BACKGROUND))
            .fill(theme::ACCENT)
            .corner_radius(23.0)
            .min_size(egui::vec2(46.0, 46.0));
        if ui
            .add_enabled(state.has_media(), play)
            .on_hover_text(hint)
            .clicked()
        {
            action = Some(ControlAction::TogglePlay);
        }

        ui.add_space(4.0);

        if state.has_media() {
            ui.label(
                egui::RichText::new(format!(
                    "{} / {}",
                    timeline::format_time(state.position),
                    timeline::format_time(state.duration)
                ))
                .monospace()
                .size(13.0)
                .color(theme::TEXT_DIM),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            export_control(ui, state, &mut action);
            ui.add_space(8.0);
            if ui.button("Open video").clicked() {
                action = Some(ControlAction::OpenMedia);
            }
            ui.add_space(8.0);
            volume_control(ui, state, &mut action);
        });
    });

    action
}

fn volume_control(ui: &mut egui::Ui, state: &AppState, action: &mut Option<ControlAction>) {
    let mut volume = state.volume;
    let slider = ui.add_sized(
        egui::vec2(96.0, 18.0),
        egui::Slider::new(&mut volume, 0.0..=100.0).show_value(false),
    );
    if slider.changed() {
        *action = Some(ControlAction::SetVolume(volume));
    }
    slider.on_hover_text(format!("Volume: {:.0}%", volume));

    let (glyph, hint) = if state.muted {
        ("🔇", "Unmute")
    } else {
        ("🔊", "Mute")
    };
    let mute = egui::Button::new(egui::RichText::new(glyph).size(15.0)).frame(false);
    if ui.add(mute).on_hover_text(hint).clicked() {
        *action = Some(ControlAction::ToggleMute);
    }
}

fn export_control(ui: &mut egui::Ui, state: &AppState, action: &mut Option<ControlAction>) {
    match &state.export {
        ExportStatus::Idle => {
            let enabled = state.has_media() && state.selection.is_usable();
            let button = theme::primary_button("✂  Export clip");
            if ui.add_enabled(enabled, button).clicked() {
                *action = Some(ControlAction::Export);
            }
        }
        ExportStatus::Running(fraction) => {
            ui.add(
                egui::ProgressBar::new(*fraction)
                    .desired_width(170.0)
                    .text(format!("{:.0}%", fraction * 100.0)),
            );
        }
        ExportStatus::Finished(path) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string());
            status_chip(ui, theme::ACCENT, "✔  Exported")
                .on_hover_text(format!("Saved as {name}"));
        }
        ExportStatus::Failed(reason) => {
            status_chip(ui, theme::DANGER, "✖  Failed").on_hover_text(reason);
        }
    }
}

fn status_chip(ui: &mut egui::Ui, fill: egui::Color32, text: &str) -> egui::Response {
    theme::chip(fill)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text.to_owned())
                    .color(theme::BACKGROUND)
                    .strong(),
            );
        })
        .response
}
