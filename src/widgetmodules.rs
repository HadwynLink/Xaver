use iced::{
    Element, Length,
    alignment::Vertical,
    widget::{Button, Column, Row, Text, button, column, combo_box, row, scrollable, text},
};
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    savedir: String,
    backupdir: String,
}

#[derive(Debug, Clone)]
enum Message {
    Save,
    Load,
    Checkpoint,
    SaveSelected(String),
    Refresh,
    Launch,
    Settings,
}

struct SaveSelector {
    tarfold: String,
    backupfold: String,
    saves: Vec<String>,
    backups: Vec<String>, // Need for cross-referencing
    combo_state: combo_box::State<String>,
    selected_save: Option<String>,
}

impl SaveSelector {
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

        let tardir = format!("{}", data.savedir);
        let savedir = format!("{}", data.backupdir);
        let saveops = savemanager::compile_saves(&tardir);
        let backops = savemanager::compile_saves(&savedir);
        Self {
            combo_state: combo_box::State::new(saveops.clone()),
            selected_save: None,
            saves: saveops,
            backups: backops,
            tarfold: tardir,
            backupfold: savedir,
        }
    }
    fn update(&self) -> Column<'_, Message> {
        match message {
            Message::SaveSelected(option) => {
                self.selected_disp = Some(option);
                if self.selected_disp != None {
                    self.selected_save = self.saves[self
                        .savedisp
                        .iter()
                        .position(|disp| disp == &self.selected_disp.clone().unwrap())
                        .unwrap()]
                    .clone();
                } else {
                    self.selected_save = format!("");
                }
            }
            Message::Refresh => {
                self.saves = savemanager::compile_saves(&self.tarfold);
                self.backups = savemanager::compile_saves(&self.backupfold);
            }
            Message::Launch => {
                Command::new("xdg-open")
                    .arg("steam://run/362490")
                    .spawn()
                    .unwrap();
            }
            _ => {}
        }
    }
    fn view(&self) -> Column<'_, Message> {
        let mut screening = column![
            text(format!("Game folder: {}", self.tarfold)),
            text(format!("Saves folder: {}", self.backupfold)),
            row![
                button("Refresh")
                    .on_press(Message::Refresh)
                    .width(Length::Fill),
                button("Launch")
                    .on_press(Message::Launch)
                    .width(Length::Fill),
                button("Settings")
                    .on_press(Message::Settings)
                    .width(Length::Fill),
            ]
            .spacing(10),
            text("Current Save:"),
            combo_box(
                &self.combo_state,
                "Choose a save...",
                self.selected_save.as_ref(),
                Message::SaveSelected
            ),
            self.info_widget.view()
        ]
        .padding(5)
        .spacing(5);
        screening
    }
}
