use crate::state::app_state::AppState;
use dioxus::prelude::*;

#[component]
pub fn PreviewPlayer() -> Element {
    let state = AppState::use_state();
    let has_video = state.video_path.read().is_some();

    rsx! {
        div {
            class: "preview-area",
            style: "background-color: transparent !important;",
            if !has_video {
                div { class: "placeholder-text", "No video loaded" }
            } else {
                div { style: "width: 100%; height: 100%; background: transparent;" }
            }
        }
    }
}
