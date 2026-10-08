use eframe::egui;

use crate::app::{AppState, ExportStatus};
use crate::ui::{theme, timeline};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAction {
    OpenMedia,
    TogglePlay,
    Export,
}

pub fn show(ui: &mut egui::Ui, state: &AppState) -> Option<ControlAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        if ui.button("Open video").clicked() {
            action = Some(ControlAction::OpenMedia);
        }

        let play_label = if state.playing { "⏸  Pause" } else { "▶  Play" };
        if ui
            .add_enabled(state.has_media(), egui::Button::new(play_label))
            .clicked()
        {
            action = Some(ControlAction::TogglePlay);
        }

        if state.has_media() {
            ui.label(
                egui::RichText::new(format!(
                    "{} / {}",
                    timeline::format_time(state.position),
                    timeline::format_time(state.duration)
                ))
                .monospace()
                .color(theme::TEXT_DIM),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            export_control(ui, state, &mut action);
        });
    });

    action
}

fn export_control(ui: &mut egui::Ui, state: &AppState, action: &mut Option<ControlAction>) {
    match &state.export {
        ExportStatus::Idle => {
            let enabled = state.has_media() && state.selection.is_usable();
            if ui
                .add_enabled(enabled, egui::Button::new("✂  Export clip"))
                .clicked()
            {
                *action = Some(ControlAction::Export);
            }
        }
        ExportStatus::Running(fraction) => {
            ui.add(
                egui::ProgressBar::new(*fraction)
                    .desired_width(160.0)
                    .text(format!("{:.0}%", fraction * 100.0)),
            );
        }
        ExportStatus::Finished(path) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string());
            ui.label(egui::RichText::new(format!("✔ {name}")).color(theme::ACCENT));
        }
        ExportStatus::Failed(reason) => {
            ui.label(egui::RichText::new(format!("✖ {reason}")).color(theme::DANGER));
        }
    }
}
