use crate::messages::*;
use crate::savemanager;
use chrono::{DateTime, Local};
use iced::{
    Color, Length, color,
    widget::{
        Column, Container, button, center_x, column, container, image, pick_list, row, rule, text,
    },
};

use std::env;
use std::f64;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

fn banner<'a>(filter_method: image::FilterMethod) -> Container<'a, Message> {
    let banpath: String = format!(
        "{}/images/Banner.png",
        env::current_dir()
            .expect("could not find current directory")
            .display()
    );
    center_x(
        image(banpath)
            .filter_method(filter_method)
            .width(Length::Fill),
    )
}

pub struct FullState {
    // 'global' variables: extremely useful
    gamefolder: String, // Folder with active saves
    savefolder: String, // Folder with saves

    // Variables for save selector
    saves: Vec<String>,            // List of active saves
    savedisp: Vec<String>,         // Display/fancy list of saves
    selected_disp: Option<String>, // Selected save (fancy)
    selected_save: String,         // Selected save
    selected_path: String,
    backups: Vec<String>, // List of backups for the selected save

    // Variables for save info
    created_date: String, // Date selected file was created
    charname: String,     // Name of selected character
    gametype: String,     // Type of game (Arena vs Campaign)
    charloc: String,      // Location of the character in-game
}

impl FullState {
    pub fn default() -> Self {
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
        let saveops = savemanager::compile_saves(&gamedir);

        Self {
            gamefolder: gamedir.clone(),
            savefolder: savedir.clone(),

            selected_disp: None,
            selected_save: String::new(),
            savedisp: savemanager::generate_save_display(&gamedir, &saveops),
            saves: saveops,

            created_date: String::new(),
            charname: String::new(),
            gametype: String::new(),
            charloc: String::new(),
            selected_path: String::new(),
            backups: Vec::new(),
        }
    }
    pub fn update(&mut self, message: Message) {
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
                    self.update_info()
                        .expect("Couldn't update save information");
                    self.selected_path = format!(
                        "{}{}",
                        self.savefolder,
                        self.selected_save.replace(".rsg", "")
                    );
                    if fs::metadata(&self.selected_path).is_ok() {
                        self.backups = savemanager::compile_saves(&self.selected_path);
                    } else {
                        self.backups = Vec::new();
                    };
                } else {
                    self.selected_save = String::new();
                }
            }
            Message::Refresh => {
                self.refresh();
            }
            Message::Launch => {
                Command::new("xdg-open")
                    .arg("steam://run/362490")
                    .spawn()
                    .unwrap();
            }
            Message::NewSave => {
                savemanager::new_save(
                    &format!("{}{}", self.gamefolder, self.selected_save),
                    &self.selected_path,
                    &format!("Save {}", self.backups.len() + 1),
                );
                self.backups = savemanager::compile_saves(&self.selected_path);
            }
            Message::OverwriteSave(tar) => {
                savemanager::copy_save(
                    &format!("{}{}", self.gamefolder, self.selected_save),
                    &format!("{}{}", &self.selected_path, tar),
                );
                self.refresh();
            }
            Message::RestoreSave(tar) => {
                savemanager::copy_save(
                    &format!("{}{}", &self.selected_path, tar),
                    &format!("{}{}", self.gamefolder, self.selected_save),
                );
                self.refresh();
            }
            Message::DeleteSave(tar) => {
                fs::remove_file(format!("{}{}", &self.selected_path, tar))
                    .expect("Couldn't delete save!");
                self.refresh();
            }
            _ => {}
        }
    }
    pub fn view(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> =
            column!(banner(image::FilterMethod::Linear), self.save_selector());
        if self.selected_disp != None {
            screening = screening.push(self.save_info());
            screening = screening.push(self.save_slots());
        }

        screening
    }

    fn refresh(&mut self) {
        self.saves = savemanager::compile_saves(&self.gamefolder);
        self.backups = savemanager::compile_saves(&self.selected_path);
    }

    // Display code for save selector
    fn save_selector(&self) -> Column<'_, Message> {
        let screening = column![
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
        screening
    }

    fn update_info(&mut self) -> io::Result<()> {
        if self.selected_disp != None {
            let fullpath: String = format!("{}{}", &self.gamefolder, &self.selected_save);
            let metadata = fs::metadata(&fullpath).unwrap();
            let created: DateTime<Local> = metadata
                .created()
                .expect("Couldn't find creation date")
                .into();
            self.created_date = format!("{}", created.format("%m/%d/%Y at %-I:%M %p"));
            self.charname = savemanager::read_name(&fullpath);
            if fullpath.contains("Arena0") {
                self.gametype = format!("Arena");
            } else {
                self.gametype = format!("Campaign");
            }
            self.charloc = savemanager::read_level(&fullpath);
        }
        Ok(())
    }

    pub fn save_info(&self) -> Column<'_, Message> {
        let screening = column![
            text!("Character Name: {}", self.charname),
            text!("Location: {}", self.charloc.replace("\n", " ")),
            //text!("Save Type: {}", self.gametype),
            text!("Created: {}", self.created_date),
        ]
        .padding(5)
        .spacing(10);
        screening
    }

    fn save_slots(&self) -> Column<'_, Message> {
        let mut screening = column![
            text!("Saves").size(25).width(Length::Fill).center(),
            rule::horizontal(2),
            button(text!("New Save").height(40).center().width(Length::Fill))
                .width(Length::Fill)
                .height(40)
                .on_press(Message::NewSave),
        ]
        .padding(5)
        .spacing(10);
        for save in &self.backups {
            screening = screening.push(self.save_slot(&save));
        }
        screening
    }

    fn save_slot(&self, tar: &String) -> Column<'_, Message> {
        let meta = fs::metadata(format!("{}{}", &self.selected_path, &tar)).unwrap();
        let time_saved: DateTime<Local> =
            meta.modified().expect("Couldn't find creation date").into();
        let file_size: f64 = (meta.size() as f64) / 1000000.0;
        column![
            container(
                row!(
                    column!(
                        text!("{}", tar.replace(".rsg", "").replace("/", "")).size(20),
                        text!("{:.2} MB", file_size)
                            .size(15)
                            .color(color!(150, 150, 150))
                    )
                    .width(220),
                    rule::vertical(2),
                    column![
                        text!(
                            "Name: {}",
                            savemanager::read_name(&format!("{}{}", &self.selected_path, &tar))
                        )
                        .height(20)
                        .center()
                        .width(Length::Fill),
                        rule::horizontal(2),
                        text!(
                            "Location: {}",
                            savemanager::read_level(&format!("{}{}", &self.selected_path, &tar))
                        )
                        .height(Length::Fill)
                        .center()
                        .width(Length::Fill)
                    ]
                    .spacing(5),
                    rule::vertical(2),
                    text!("Saved {}", time_saved.format("%m/%d/%Y at %-I:%M %p"))
                        .height(75)
                        .center()
                        .width(Length::Fill),
                    rule::vertical(2),
                    button("Save")
                        .on_press(Message::OverwriteSave(tar.clone()))
                        .width(75)
                        .height(75),
                    button("Restore")
                        .on_press(Message::RestoreSave(tar.clone()))
                        .width(75)
                        .height(75),
                    button("Delete")
                        .on_press(Message::DeleteSave(tar.clone()))
                        .width(75)
                        .height(75),
                )
                .align_y(iced::Alignment::Center)
                .spacing(10)
            )
            .align_y(iced::Alignment::Center)
            .padding(10)
            .width(Length::Fill)
            .height(85)
            .style(container::rounded_box),
        ]
    }
}
