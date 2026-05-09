use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

mod app;
mod player;
mod state;
mod ui;
mod video;

use app::App;

fn main() {
    let cfg = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_transparent(true)
                .with_title("Trimontina"),
        )
        .with_background_color((0, 0, 0, 0))
        .with_menu(None);

    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}
