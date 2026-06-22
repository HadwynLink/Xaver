// Purpose: To save and load exanima games easily, and create checkpoints artificially
mod messages;
mod savemanager;
mod utils;
mod windowdisplay;
use iced::{Application, window};

pub fn main() -> iced::Result {
    iced::application(
        windowdisplay::Xaver::default,
        windowdisplay::Xaver::update,
        windowdisplay::Xaver::view,
    )
    .window(window::Settings {
        icon: utils::load_icon().unwrap_or(None),
        ..Default::default()
    })
    .run()
}
