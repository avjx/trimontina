mod selection;
mod state;

pub use state::{AppState, DragTarget, ExportStatus};

use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError},
};

use eframe::egui;

use crate::player::{Player, PlayerError};
use crate::ui::{self, controls::ControlAction};
use crate::video::{self, ExportEvent};

// After a seek or a load, keep repainting so mpv keeps being polled until the
// new frame is ready. Timed, not counted: repaint cycles are far faster than
// mpv's decode.
const SETTLE: std::time::Duration = std::time::Duration::from_millis(2000);

/// sec the overlay takes to fade in or out. distinct from `IDLE_LIMIT`,
/// which is how long the UI waits before fading
const FADE_SECONDS: f32 = 0.40;

pub struct TrimontinaApp {
    player: Player,
    state: AppState,
    repaint_hooked: bool,
    pending_action: Option<ControlAction>,
    pending_seek: Option<f64>,
    media_dialog: Option<Receiver<PathBuf>>,
    export_events: Option<Receiver<ExportEvent>>,
    settle_until: Option<std::time::Instant>,
}

impl TrimontinaApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_media: Option<PathBuf>,
    ) -> Result<Self, PlayerError> {
        ui::theme::apply(&cc.egui_ctx);

        let mut app = Self {
            player: Player::new(cc)?,
            state: AppState::default(),
            repaint_hooked: false,
            pending_action: None,
            pending_seek: None,
            media_dialog: None,
            export_events: None,
            settle_until: None,
        };

        if let Some(path) = initial_media {
            app.load_media(path);
        }

        Ok(app)
    }

    fn apply_pending(&mut self) {
        if let Some(seconds) = self.pending_seek.take() {
            self.state.position = seconds;
            self.settle();
            if let Err(err) = self.player.seek(seconds) {
                self.report(err);
            }
        }

        match self.pending_action.take() {
            Some(ControlAction::OpenMedia) => self.open_media_dialog(),
            Some(ControlAction::TogglePlay) => self.toggle_play(),
            Some(ControlAction::Export) => self.start_export(),
            Some(ControlAction::SetVolume(volume)) => {
                self.state.volume = volume;
                if let Err(err) = self.player.set_volume(f64::from(volume)) {
                    self.report(err);
                }
            }
            Some(ControlAction::ToggleMute) => {
                let muted = !self.state.muted;
                self.state.muted = muted;
                if let Err(err) = self.player.set_muted(muted) {
                    self.report(err);
                }
            }
            None => {}
        }
    }

    fn open_media_dialog(&mut self) {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.media_dialog = Some(receiver);

        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Video", &["mp4", "mkv", "avi", "mov", "webm"])
                .pick_file()
            {
                let _ = sender.send(path);
            }
        });
    }

    fn poll_media_dialog(&mut self) {
        let Some(receiver) = self.media_dialog.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(path) => self.load_media(path),
            Err(TryRecvError::Empty) => self.media_dialog = Some(receiver),
            Err(TryRecvError::Disconnected) => {}
        }
    }

    fn load_media(&mut self, path: PathBuf) {
        self.settle();
        match self.player.load(&path) {
            Ok(()) => self.state.set_media(path),
            Err(err) => self.report(err),
        }
    }

    fn toggle_play(&mut self) {
        self.settle();

        if self.player.is_eof() {
            if let Err(err) = self.player.seek(0.0) {
                self.report(err);
            }
            self.state.position = 0.0;
        }

        let resuming = self.player.is_paused();
        let result = if resuming {
            self.player.play()
        } else {
            self.player.pause()
        };
        if let Err(err) = result {
            self.report(err);
        }
        self.state.playing = resuming;
        self.state.mark_activity();
    }

    fn start_export(&mut self) {
        let Some(input) = self.state.media_path.clone() else {
            return;
        };
        if !self.state.selection.is_usable() {
            self.state.error = Some("select a clip longer than zero".to_string());
            return;
        }

        let Some(output) = rfd::FileDialog::new()
            .set_file_name("clip.mp4")
            .save_file()
        else {
            return;
        };

        let cut = video::Cut {
            start: self.state.selection.start(),
            end: self.state.selection.end(),
        };
        self.export_events = Some(video::cut_in_background(input, output, cut));
        self.state.export = ExportStatus::Running(0.0);
        self.state.export_result_at = None;
        self.state.error = None;
    }

    fn poll_export(&mut self) {
        let Some(receiver) = self.export_events.take() else {
            return;
        };

        let mut finished = false;
        loop {
            match receiver.try_recv() {
                Ok(ExportEvent::Progress(fraction)) => {
                    self.state.export = ExportStatus::Running(fraction);
                }
                Ok(ExportEvent::Completed(path)) => {
                    self.state.finish_export(ExportStatus::Finished(path));
                    finished = true;
                }
                Ok(ExportEvent::Failed(reason)) => {
                    self.state.error = Some(reason.clone());
                    self.state.finish_export(ExportStatus::Failed(reason));
                    finished = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    finished = true;
                    break;
                }
            }
        }

        if !finished {
            self.export_events = Some(receiver);
        }
    }

    fn sync_from_player(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.pointer.delta()) != egui::Vec2::ZERO {
            self.state.mark_activity();
        }

        if !self.state.has_media() {
            return;
        }

        self.state.observe_duration(self.player.duration());

        if !self.state.scrubbing {
            self.state.position = self.player.position();
        }
        self.state.playing = !self.player.is_paused();

        if self.player.is_eof() {
            let _ = self.player.pause();
            let _ = self.player.seek(0.0);
            self.state.playing = false;
            self.state.position = 0.0;
        }

        self.state.tick(ctx.input(|i| i.stable_dt));
    }

    fn settle(&mut self) {
        self.settle_until = Some(std::time::Instant::now() + SETTLE);
    }

    fn report(&mut self, error: PlayerError) {
        self.state.error = Some(error.to_string());
    }
}

impl eframe::App for TrimontinaApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Err(err) = self.player.ensure_surface(frame) {
            self.report(err);
            return;
        }

        if !self.repaint_hooked {
            self.player.repaint_on_new_frames(ctx.clone());
            self.repaint_hooked = true;
        }

        self.apply_pending();
        self.poll_media_dialog();
        self.poll_export();
        self.state.expire_export(std::time::Instant::now());
        if let Some(remaining) = self.state.export_result_remaining() {
            ctx.request_repaint_after(remaining);
        }
        self.sync_from_player(ctx);

        if let Err(err) = self.player.render() {
            self.report(err);
        }

        //mpv drops frames when the embedder draws too rarely.
        let playing = !self.player.is_paused();
        let settling =
            matches!(self.settle_until, Some(deadline) if std::time::Instant::now() < deadline);
        if self.settle_until.is_some() && !settling {
            self.settle_until = None;
        }
        if playing || settling {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Self {
            player,
            state,
            pending_action,
            pending_seek,
            ..
        } = self;

        let ctx = ui.ctx().clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(ui::theme::BACKGROUND))
            .show_inside(ui, |ui| {
                if ui::preview::show(ui, player, state.has_media()) {
                    *pending_action = Some(ControlAction::TogglePlay);
                }
            });

        let opacity = ctx.animate_bool_with_time(
            egui::Id::new("trimontina-overlay"),
            state.ui_visible(),
            FADE_SECONDS,
        );
        if opacity <= 0.01 {
            return;
        }
        if opacity < 0.99 {
            ctx.request_repaint();
        }

        let screen = ctx.content_rect();
        let margin = 18.0;
        let width = (screen.width() - margin * 2.0).max(320.0);

        egui::Area::new(egui::Id::new("trimontina-header"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(margin, margin))
            .interactable(state.ui_visible())
            .show(&ctx, |ui| {
                ui.set_width(width);
                ui::theme::card(opacity).show(ui, |ui| header_bar(ui, state));
            });

        egui::Area::new(egui::Id::new("trimontina-controls"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(margin, -margin))
            .interactable(state.ui_visible())
            .show(&ctx, |ui| {
                ui.set_width(width);
                ui::theme::card(opacity).show(ui, |ui| {
                    bottom_bar(ui, state, pending_action, pending_seek)
                });
            });
    }
}

fn header_bar(ui: &mut egui::Ui, state: &AppState) {
    ui.horizontal(|ui| {
        let exporting = matches!(state.export, ExportStatus::Running(_));
        ui::title::glitch_title(ui, "TRIMONTINA", exporting);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let name = state
                .media_path
                .as_ref()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "No file".to_string());
            ui.label(
                egui::RichText::new(name)
                    .color(ui::theme::TEXT_DIM)
                    .size(12.0),
            );
        });
    });
}

fn bottom_bar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    pending_action: &mut Option<ControlAction>,
    pending_seek: &mut Option<f64>,
) {
    let action = ui::timeline::show(ui, state);
    if action.seek_to.is_some() {
        *pending_seek = action.seek_to;
    }

    ui.add_space(8.0);

    if let Some(action) = ui::controls::show(ui, state) {
        *pending_action = Some(action);
    }

    if let Some(text) = &state.error {
        ui.colored_label(ui::theme::DANGER, text);
    }
}
