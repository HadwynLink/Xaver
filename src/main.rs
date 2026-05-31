// Purpose: To save and load exanima games easily, and create checkpoints artificially
mod messages;
mod savemanager;
mod widgetmodules;
use iced::widget::Column;
use iced_aw::widget;
use messages::*;
use std::env;
use std::fs;

struct FullState {
    gamefolder: String,
    savefolder: String,

    select_widget: widgetmodules::SaveSelector,
    save_info: widgetmodules::SaveInfo,
    save_slots: widgetmodules::SaveSlot,
}

impl FullState {
    fn default() -> Self {
        let cfgpath: String = format!(
            "{}/config/config.json",
            env::current_dir()
                .expect("could not find current directory")
                .display()
        );
        let config = fs::read_to_string(cfgpath).expect("Config error: could not find config.json");
        let data: Config =
            serde_json::from_str(&config).expect("Config error: could not parse file");

        let gamedir = format!("{}", data.savedir);
        let savedir = format!("{}", data.backupdir);

        Self {
            select_widget: widgetmodules::SaveSelector::default(&gamedir, &savedir),
            save_info: widgetmodules::SaveInfo::default(),
            save_slots: widgetmodules::SaveSlot::default(),

            gamefolder: gamedir,
            savefolder: savedir,
        }
    }
    fn update(&mut self, message: Message) {
        self.select_widget.update(message);
        if self.select_widget.selected_save != "" {
            self.save_info
                .update_info(&self.gamefolder, &self.select_widget.selected_save)
                .expect("error updating display");
        }
        match message {
            _ => {}
        }
    }
    fn view(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> = self.select_widget.view().into();
        if self.select_widget.selected_save != "" {
            screening = screening.push(self.save_info.view());
        }
        screening = screening.push(self.save_slots.view());

        screening
    }
}

pub fn main() -> iced::Result {
    iced::application(FullState::default, FullState::update, FullState::view).run()
}
