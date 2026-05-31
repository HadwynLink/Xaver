use crate::messages::*;
use crate::savemanager;
use chrono::{DateTime, Local};
use iced::{
    Length,
    widget::{Column, button, column, pick_list, row, text},
};
use std::fs;
use std::io;
use std::process::Command;

// Selector for save slots
pub struct SaveSelector {
    tarfold: String,               // Game folder
    backupfold: String,            // Backups folder
    saves: Vec<String>,            // List of game saves
    backups: Vec<String>,          // Need for cross-referencing
    savedisp: Vec<String>,         // Display/fancy list of saves
    selected_disp: Option<String>, // Selected save (fancy)
    pub selected_save: String,     // Selected save
}
impl SaveSelector {
    pub fn default(gamedir: &String, savedir: &String) -> Self {
        let saveops = savemanager::compile_saves(gamedir);
        let backops = savemanager::compile_saves(savedir);
        Self {
            selected_disp: None,
            selected_save: String::new(),
            savedisp: savemanager::generate_save_display(gamedir, &saveops),
            saves: saveops,
            backups: backops,
            tarfold: format!("{}", gamedir),
            backupfold: format!("{}", savedir),
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
    pub fn view(&self) -> Column<'_, Message> {
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
}

// Information about the current save
pub struct SaveInfo {
    created_date: String,
    charname: String,
    gametype: String,
    charloc: String,
}
impl SaveInfo {
    pub fn default() -> Self {
        Self {
            created_date: String::new(),
            charname: String::new(),
            gametype: String::new(),
            charloc: String::new(),
        }
    }
    pub fn update_info(&mut self, cursave: &String, curtar: &String) -> io::Result<()> {
        if cursave != "" {
            let mut fullpath: String = cursave.clone();
            fullpath.push_str(&curtar);
            let metadata = fs::metadata(&fullpath).unwrap();
            let created: DateTime<Local> = metadata.created()?.into();
            self.created_date = format!("{}", created.format("%m/%d/%Y at %-I:%M %p"));
            self.charname = savemanager::read_name(cursave, curtar);
            if curtar.contains("Arena") {
                self.gametype = format!("Arena");
            } else {
                self.gametype = format!("Campaign");
            }
            self.charloc = savemanager::read_level(cursave, curtar);
        }
        Ok(())
    }
    pub fn view(&self) -> Column<'_, Message> {
        let screening = column![
            text!("Character Name: {}", self.charname),
            text!("Location: {}", self.charloc),
            //text!("Save Type: {}", self.gametype),
            text!("Created: {}", self.created_date),
        ]
        .padding(5)
        .spacing(10);
        screening
    }
}

//
pub struct SaveSlot {
    // Save name
    // Date modified
    // Player location
    // File size
}
impl SaveSlot {
    pub fn default() -> Self {
        Self {}
    }
    pub fn view(&self) -> Column<'_, Message> {
        let screening = column![text!("Save Slot:"),].padding(5).spacing(10);
        screening
    }
}
