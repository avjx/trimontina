use std::path::PathBuf;

use super::selection::Selection;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DragTarget {
    #[default]
    None,
    Start,
    End,
    Playhead,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ExportStatus {
    #[default]
    Idle,
    Running(f32),
    Finished(PathBuf),
    Failed(String),
}

#[derive(Debug)]
pub struct AppState {
    pub media_path: Option<PathBuf>,
    pub selection: Selection,
    pub duration: f64,
    pub position: f64,
    pub playing: bool,
    pub scrubbing: bool,
    pub drag_target: DragTarget,
    pub idle_seconds: f32,
    pub export: ExportStatus,
    pub error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            media_path: None,
            selection: Selection::full(0.0),
            duration: 0.0,
            position: 0.0,
            playing: false,
            scrubbing: false,
            drag_target: DragTarget::None,
            idle_seconds: 0.0,
            export: ExportStatus::Idle,
            error: None,
        }
    }
}

impl AppState {
    const IDLE_LIMIT: f32 = 2.5;

    pub fn set_media(&mut self, path: PathBuf) {
        self.media_path = Some(path);
        self.duration = 0.0;
        self.position = 0.0;
        self.playing = false;
        self.selection = Selection::full(0.0);
        self.export = ExportStatus::Idle;
        self.error = None;
    }

    pub fn observe_duration(&mut self, duration: f64) {
        if duration <= 0.0 || (self.duration - duration).abs() < f64::EPSILON {
            return;
        }
        self.duration = duration;
        if self.selection.media_duration() <= 0.0 {
            self.selection = Selection::full(duration);
        }
    }

    pub fn ui_visible(&self) -> bool {
        self.idle_seconds < Self::IDLE_LIMIT
    }

    pub fn has_media(&self) -> bool {
        self.media_path.is_some()
    }

    pub fn mark_activity(&mut self) {
        self.idle_seconds = 0.0;
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        if self.playing && !self.scrubbing {
            self.idle_seconds += delta_seconds;
        } else {
            self.idle_seconds = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use std::path::PathBuf;

    #[test]
    fn opening_media_resets_derived_state() {
        let mut state = AppState {
            duration: 42.0,
            position: 10.0,
            ..AppState::default()
        };

        state.set_media(PathBuf::from("/tmp/a.mp4"));

        assert_eq!(state.duration, 0.0);
        assert_eq!(state.position, 0.0);
        assert!(!state.playing);
        assert!(state.has_media());
    }

    #[test]
    fn discovered_duration_adjusts_selection_once() {
        let mut state = AppState::default();
        state.observe_duration(12.0);
        assert_eq!(state.selection.end(), 12.0);

        state.selection.set_end(4.0);
        state.observe_duration(12.0);
        assert_eq!(state.selection.end(), 4.0);
    }

    #[test]
    fn ui_hides_only_while_playing() {
        let mut state = AppState {
            playing: true,
            ..AppState::default()
        };
        state.tick(10.0);
        assert!(!state.ui_visible());

        state.mark_activity();
        assert!(state.ui_visible());
    }
}
