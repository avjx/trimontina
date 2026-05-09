use crate::state::app_state::{AppState, DragTarget};
use crate::ui::controls::Controls;
use crate::ui::timeline::Timeline;
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use rfd::AsyncFileDialog;
use std::time::Duration;

#[component]
pub fn App() -> Element {
    let window = use_window();
    let wid = match window.window_handle().unwrap().as_raw() {
        RawWindowHandle::Win32(handle) => handle.hwnd.get().to_string(),
        RawWindowHandle::Xlib(handle) => handle.window.to_string(),
        RawWindowHandle::Wayland(handle) => format!("{}", handle.surface.as_ptr() as usize),
        _ => "0".to_string(),
    };

    AppState::init(wid);
    let mut state = AppState::use_state();
    let dragging = (state.dragging)();

    let mut ui_visible = use_signal(|| true);
    let mut idle_ms = use_signal(|| 0u32);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let is_playing = *state.is_playing.peek();
            let is_dragging = *state.dragging.peek() != DragTarget::None;

            if !is_playing || is_dragging {
                idle_ms.set(0);
                ui_visible.set(true);
            } else {
                idle_ms.with_mut(|ms| *ms += 100);
                if ui_visible() && idle_ms() > 2500 {
                    ui_visible.set(false);
                }
            }

            if state.video_path.read().is_some() {
                let mpv = state.mpv.peek();
                if state.duration.peek().abs() < 0.1 {
                    let dur = mpv.get_duration();
                    if dur > 0.1 {
                        state.duration.set(dur);
                    }
                }

                if !is_dragging {
                    let cur = mpv.get_current_time();
                    let dur = *state.duration.peek();
                    if dur > 0.0 {
                        let pct = (cur / dur) * 100.0;
                        state.current_time.set(pct);
                        if pct > 99.8 {
                            state.is_playing.set(false);
                            mpv.seek(0.0);
                        }
                    }
                }
            }
        }
    });

    let ui_class = if ui_visible() {
        "app-container ui-visible"
    } else {
        "app-container ui-hidden"
    };

    rsx! {
        style { {include_str!("../assets/main.css")} }

        div {
            style: "position: fixed; width: 100vw; height: 100vh; background: transparent; z-index: 1; cursor: default;",
            onmousemove: move |_| {
                ui_visible.set(true);
                idle_ms.set(0);
            },
            onclick: move |_| {
                if state.video_path.read().is_some() {
                    let next = !(state.is_playing)();
                    state.is_playing.set(next);
                    state.mpv.peek().set_pause(!next);
                }
            }
        }

        div {
            class: "{ui_class}",
            style: "z-index: 10;",

            if dragging != DragTarget::None {
                div {
                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 9999; cursor: ew-resize; background: transparent; pointer-events: auto;",
                    onmousemove: move |evt| {
                        spawn(async move {
                            let mut eval_js = document::eval(r#"
                                let el = document.querySelector('.timeline-track');
                                if (el) {
                                    let rect = el.getBoundingClientRect();
                                    dioxus.send(JSON.stringify({
                                        left: rect.left,
                                        width: rect.width
                                    }));
                                }
                            "#);
                            if let Ok(json_val) = eval_js.recv::<serde_json::Value>().await {
                                if let Some(json_str) = json_val.as_str() {
                                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                                        let left = data["left"].as_f64().unwrap_or(0.0);
                                        let width = data["width"].as_f64().unwrap_or(1.0);
                                        let client_x = evt.data().client_coordinates().x;
                                        let pct = ((client_x - left) / width * 100.0).clamp(0.0, 100.0);

                                        match dragging {
                                            DragTarget::Playhead => {
                                                state.current_time.set(pct);
                                                state.mpv.peek().seek((pct / 100.0) * *state.duration.peek());
                                            }
                                            DragTarget::Start => if pct < *state.selection_end.peek() { state.selection_start.set(pct); },
                                            DragTarget::End => if pct > *state.selection_start.peek() { state.selection_end.set(pct); },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        });
                    },
                    onmouseup: move |_| state.dragging.set(DragTarget::None),
                }
            }

            header {
                h1 { "Trimontina" }
                button {
                    class: "secondary",
                    onclick: move |_| {
                        spawn(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter("Video", &["mp4", "mkv", "avi", "mov"])
                                .pick_file()
                                .await;
                            if let Some(file) = file {
                                let path = file.path().to_path_buf();
                                state.video_path.set(Some(path.clone()));
                                state.duration.set(0.0);
                                state.mpv.peek().load_file(path);
                            }
                        });
                    },
                    if let Some(path) = state.video_path.read().as_ref() {
                        "{path.file_name().unwrap().to_string_lossy()}"
                    } else {
                        "Open Video"
                    }
                }
            }

            main { class: "main-content" }

            div { class: "timeline-container",
                Timeline {}
                Controls {}
            }
        }
    }
}
