// Purpose: To save and load exanima games easily, and create checkpoints artificially
mod messages;
mod savemanager;
mod utils;
mod windowdisplay;
//mod widgetmodules;

pub fn main() -> iced::Result {
    iced::application(
        windowdisplay::Xaver::default,
        windowdisplay::Xaver::update,
        windowdisplay::Xaver::view,
    )
    .run()
}
