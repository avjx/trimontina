use crate::state::app_state::{AppState, DragTarget};
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn Timeline() -> Element {
    let mut state = AppState::use_state();
    let mut timeline_element = use_signal::<Option<Rc<MountedData>>>(|| None);

    let update_pos = move |evt: MouseEvent, target: DragTarget| {
        let client_x = evt.data().client_coordinates().x;

        if let Some(mounted) = timeline_element.read().as_ref() {
            let mounted = mounted.clone();
            spawn(async move {
                if let Ok(rect) = mounted.get_client_rect().await {
                    let width = rect.width();
                    let left = rect.origin.x;
                    if width > 0.0 {
                        let local_x = client_x - left;
                        let pct = (local_x / width) * 100.0;
                        let pct = pct.clamp(0.0, 100.0);

                        match target {
                            DragTarget::Playhead => {
                                state.current_time.set(pct);
                                let seconds = (pct / 100.0) * *state.duration.peek();
                                state.mpv.peek().seek(seconds);
                            }
                            DragTarget::Start => {
                                if pct < *state.selection_end.peek() {
                                    state.selection_start.set(pct);
                                }
                            }
                            DragTarget::End => {
                                if pct > *state.selection_start.peek() {
                                    state.selection_end.set(pct);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
    };

    let sel_start = (state.selection_start)();
    let sel_end = (state.selection_end)();
    let current_time = (state.current_time)();
    let dragging = (state.dragging)();

    rsx! {
        div {
            class: "timeline-track",
            onmounted: move |evt| timeline_element.set(Some(evt.data())),

            onmousedown: move |evt| {
                update_pos(evt, DragTarget::Playhead);
                state.dragging.set(DragTarget::Playhead);
            },

            div {
                class: "selection-range",
                style: "left: {sel_start}%; width: {sel_end - sel_start}%",

                div {
                    class: "handle handle-start",
                    onmousedown: move |evt| {
                        evt.stop_propagation();
                        state.dragging.set(DragTarget::Start);
                    }
                }

                div {
                    class: "handle handle-end",
                    onmousedown: move |evt| {
                        evt.stop_propagation();
                        state.dragging.set(DragTarget::End);
                    }
                }
            }

            div {
                class: "playhead",
                style: "left: {current_time}%",
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    state.dragging.set(DragTarget::Playhead);
                }
            }
        }
    }
}
