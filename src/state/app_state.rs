use dioxus::prelude::*;
use std::path::PathBuf;
use crate::player::mpv::MpvController;

#[derive(Clone, Copy, PartialEq)]
pub enum DragTarget {
    None,
    Playhead,
    Start,
    End,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub video_path: Signal<Option<PathBuf>>,
    pub duration: Signal<f64>,
    pub current_time: Signal<f64>,
    pub selection_start: Signal<f64>,
    pub selection_end: Signal<f64>,
    pub is_playing: Signal<bool>,
    pub mpv: Signal<MpvController>,
    pub dragging: Signal<DragTarget>,
}

impl AppState {
    pub fn init(wid: String) {
        use_hook(|| {
            let mut mpv = MpvController::new();
            mpv.start(wid);

            let state = Self {
                video_path: Signal::new(None),
                duration: Signal::new(0.0),
                current_time: Signal::new(0.0),
                selection_start: Signal::new(0.0),
                selection_end: Signal::new(100.0),
                is_playing: Signal::new(false),
                mpv: Signal::new(mpv),
                dragging: Signal::new(DragTarget::None),
            };
            provide_context(state);
        });
    }

    pub fn use_state() -> Self {
        use_context::<AppState>()
    }
}
