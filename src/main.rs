// Purpose: To save and load exanima games easily, and create checkpoints artificially
mod savemanager;
use iced::{
    Element, Fill, Length,
    alignment::Vertical,
    widget::{
        Button, Column, Row, Text, button, column, combo_box, pick_list, row, scrollable, text,
    },
};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io;
use std::process::Command;
use std::{fmt::Display, option};

#[derive(Default)]
struct SaveOptions {
    dir_game: String,
    dir_save: String,
    tar_save: String,
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

#[derive(Debug, Deserialize)]
struct Config {
    savedir: String,
    backupdir: String,
}

#[derive(Default)]
struct SaveInfo {
    name: String,
    level: String,
    date_created: String,
    date_saved: String,
}

impl SaveInfo {
    fn view(&self) -> Column<'_, Message> {
        let mut screening = column![
            text("Name: Test"),       // Can get!
            text("Save Type: Gamer"), // Can get
            text("Level on: Level beans"),
            text("Created: Ferbuary the thirtyfifth, 2026"), // Can get
            text("Last saved: March the greathammer, 2011")  // Can get
        ];
        screening
    }
}

struct FullState {
    tarfold: String,
    backupfold: String,
    saves: Vec<String>,
    backups: Vec<String>, // Need for cross-referencing

    savedisp: Vec<String>,
    selected_disp: Option<String>,
    selected_save: String,
    save_name: String,
    info_widget: SaveInfo,
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

        let tardir = format!("{}", data.savedir);
        let savedir = format!("{}", data.backupdir);
        let saveops = savemanager::compile_saves(&tardir);
        let backops = savemanager::compile_saves(&savedir);
        Self {
            selected_disp: None,
            selected_save: String::new(),
            save_name: format!(""),
            savedisp: savemanager::generate_save_display(&tardir, &saveops),
            saves: saveops,
            backups: backops,
            tarfold: tardir,
            backupfold: savedir,
            info_widget: SaveInfo::default(),
        }
    }
    fn update(&mut self, message: Message) {
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
            Message::Save => {
                if self.selected_disp != None {
                    savemanager::copy_save(&self.tarfold, &self.backupfold, &self.selected_save);
                }
                self.backups = savemanager::compile_saves(&self.backupfold);
            }
            Message::Load => {
                if self.selected_disp != None {
                    savemanager::copy_save(&self.backupfold, &self.tarfold, &self.selected_save);
                }
            }
            Message::Checkpoint => {
                if self.selected_disp != None {
                    savemanager::checkpointify(&self.tarfold, &self.selected_save);
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
            //text(format!("Game folder: {}", self.tarfold)),
            //text(format!("Saves folder: {}", self.backupfold)),
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
            row!(
                text("Current Save:").height(32).center(),
                pick_list(
                    self.savedisp.clone(),
                    self.selected_disp.clone(),
                    Message::SaveSelected
                )
                .width(Length::Fill),
            )
            .spacing(10),
        ]
        .padding(5)
        .spacing(10);
        if self.selected_disp != None {
            screening = screening.push(
                column![
                    button("Save").on_press(Message::Save).width(150),
                    if self.backups.contains(&self.selected_save) {
                        row!(button("Load").on_press(Message::Load).width(150))
                    } else {
                        row!(
                            button("Load").width(150),
                            text("There isn't a save file to load yet.")
                        )
                    },
                    if !self.selected_save.contains("Arena") {
                        row!(
                            button("Checkpoint")
                                .on_press(Message::Checkpoint)
                                .width(150)
                        )
                    } else {
                        row!(
                            button("Checkpoint").width(150),
                            text("You cannot make checkpoints for Arena saves.")
                        )
                    },
                    if self.backups.contains(&self.selected_save) {
                        row!(button("Delete").on_press(Message::Load).width(150))
                    } else {
                        row!(
                            button("Delete").width(150),
                            text("There isn't a save file to delete yet.")
                        )
                    },
                ]
                .spacing(7),
            );
            /*let mut option_text = format!(
                "Options:
            \nSave: Backs up the current save into the saves directory.
            \nLoad: Loads the saved backup back into the game."
            );
            if !self.selected_save.clone().unwrap().contains("Arena") {
                option_text
                    .push_str("\n\nCheckpoint: Sets your character to be restored when you die.");
            }
            screening = screening.push(text(option_text));*/
        }

        screening
    }
}

pub fn main() -> iced::Result {
    iced::application(FullState::default, FullState::update, FullState::view).run()
}
