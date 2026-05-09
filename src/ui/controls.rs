use crate::state::app_state::AppState;
use crate::video::ffmpeg::cut_video;
use dioxus::prelude::*;
use rfd::AsyncFileDialog;

#[component]
pub fn Controls() -> Element {
    let mut state = AppState::use_state();
    let is_playing = (state.is_playing)();

    rsx! {
        div { class: "controls",
            button {
                onclick: move |_| {
                    let next_state = !is_playing;
                    state.is_playing.set(next_state);
                    state.mpv.peek().set_pause(!next_state);
                },
                if is_playing { "PAUSE" } else { "PLAY" }
            }
            div { style: "flex: 1" }

            button {
                class: "primary",
                onclick: move |_| {
                    spawn(async move {
                        if let Some(input_path) = state.video_path.read().clone() {
                            let duration = *state.duration.peek();
                            let start_pct = *state.selection_start.peek();
                            let end_pct = *state.selection_end.peek();

                            let start_s = (start_pct / 100.0) * duration;
                            let end_s = (end_pct / 100.0) * duration;

                            let file = AsyncFileDialog::new()
                                .set_file_name("output.mp4")
                                .add_filter("video", &["mp4"])
                                .save_file()
                                .await;

                            if let Some(file) = file {
                                let output_path = file.path().to_path_buf();
                                println!("exporting: {:.2}s até {:.2}s", start_s, end_s);

                                match cut_video(input_path, output_path, start_s, end_s) {
                                    Ok(_) => println!("success!"),
                                    Err(e) => eprintln!("error: {}", e),
                                }
                            }
                        }
                    });
                },
                "FINISH"
            }
        }
    }
}
