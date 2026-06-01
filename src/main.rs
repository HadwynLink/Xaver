// Purpose: To save and load exanima games easily, and create checkpoints artificially
mod messages;
mod savemanager;
mod widgetmodules;

pub fn main() -> iced::Result {
    iced::application(
        widgetmodules::FullState::default,
        widgetmodules::FullState::update,
        widgetmodules::FullState::view,
    )
    .run()
}
