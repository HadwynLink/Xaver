use crate::messages::*;
use crate::savemanager;
use chrono::{DateTime, Local};
use iced::{
    Length, Task, color,
    widget::{
        Column, Container, button, center_x, column, container, image, pick_list, row, rule,
        scrollable, text, text_input,
    },
};
use std::env;
use std::f64;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

fn create_img<'a>(path: &String, filter_method: image::FilterMethod) -> Container<'a, Message> {
    let banpath: String = format!(
        "{}/images/{}",
        env::current_dir()
            .expect("could not find current directory")
            .display(),
        path
    );
    center_x(image(banpath).filter_method(filter_method))
}

pub struct FullState {
    // 'global' variables: extremely useful
    cfg_path: String,
    gamefolder: String, // Folder with active saves
    savefolder: String, // Folder with saves

    // Variables for save selector
    saves: Vec<String>,            // List of active saves
    savedisp: Vec<String>,         // Display/fancy list of saves
    selected_disp: Option<String>, // Selected save (fancy)
    selected_save: String,         // Selected save
    selected_path: String,
    backups: Vec<String>, // List of backups for the selected save
    showsettings: bool,

    // Variables for save info
    created_date: String, // Date selected file was created
    charname: String,     // Name of selected character
    gametype: String,     // Type of game (Arena vs Campaign)
    charloc: String,      // Location of the character in-game
    newsavename: String,
    defaultsavename: String,

    save_slot_info: Vec<SaveInfo>, // Cache for save information

    potgamefolder: String,
    potsavefolder: String,
}

impl FullState {
    pub fn default() -> Self {
        let cfgpath: String = format!(
            "{}/config/config.json",
            env::current_dir()
                .expect("could not find current directory")
                .display()
        );
        let config =
            fs::read_to_string(&cfgpath).expect("Config error: could not find config.json");
        let data: Config =
            serde_json::from_str(&config).expect("Config error: could not parse file");

        let gamedir = format!("{}", data.savedir);
        let savedir = format!("{}", data.backupdir);
        let saveops = savemanager::compile_saves(&gamedir);

        let mut saveinfo = Vec::new();
        for save in &saveops {
            if fs::metadata(format!("{}/{}", &savedir, save.replace(".rsg", ""))).is_ok() {
                let backups = savemanager::compile_saves(&format!(
                    "{}/{}",
                    &savedir,
                    save.replace(".rsg", "")
                ));
                for backup in backups {
                    saveinfo.push(savemanager::read_info(&format!(
                        "{}/{}{}",
                        &savedir,
                        save.replace(".rsg", ""),
                        backup
                    )));
                }
            }
        }

        Self {
            cfg_path: cfgpath,
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
            newsavename: String::new(),
            defaultsavename: String::new(),

            save_slot_info: saveinfo,
            showsettings: false,
            potgamefolder: String::new(),
            potsavefolder: String::new(),
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
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
                        "{}/{}",
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
                self.findunusedsavename(self.backups.len() as i64 + 1);
                Task::none()
            }
            Message::Refresh => {
                self.refresh();
                self.findunusedsavename(self.backups.len() as i64 + 1);
                Task::none()
            }
            Message::Launch => {
                Command::new("xdg-open")
                    .arg("steam://run/362490")
                    .spawn()
                    .unwrap();
                Task::none()
            }
            Message::NewSave => {
                let savename = if self.newsavename != "" {
                    &self.newsavename
                } else {
                    &self.defaultsavename
                };
                savemanager::new_save(
                    &format!("{}{}", self.gamefolder, self.selected_save),
                    &self.selected_path,
                    savename,
                );
                self.save_slot_info.push(savemanager::read_info(&format!(
                    "{}/{}.rsg",
                    &self.selected_path, savename
                )));
                self.newsavename.clear();
                self.backups = savemanager::compile_saves(&self.selected_path);
                self.findunusedsavename(self.backups.len() as i64 + 1);
                Task::none()
            }
            Message::OverwriteSave(tar) => {
                savemanager::copy_save(
                    &format!("{}{}", self.gamefolder, self.selected_save),
                    &self.save_slot_info[tar].path,
                );
                self.refresh();
                Task::none()
            }
            Message::RestoreSave(tar) => {
                savemanager::copy_save(
                    &self.save_slot_info[tar].path,
                    &format!("{}{}", self.gamefolder, self.selected_save),
                );
                self.update_info().expect("Could not update info");
                self.refresh();
                Task::none()
            }
            Message::DeleteSave(tar) => {
                fs::remove_file(&self.save_slot_info[tar].path).expect("Couldn't delete save!");
                self.save_slot_info.remove(tar);
                self.refresh();
                self.findunusedsavename(self.backups.len() as i64 + 1);
                Task::none()
            }
            Message::ContentChanged(savename) => {
                self.newsavename = savename;
                Task::none()
            }
            Message::Settings => {
                self.showsettings = !self.showsettings;
                Task::none()
            }
            Message::OpenFolder(foldertype) => self.open_folder(foldertype),
            Message::FolderSelected(folder_type, path) => {
                if folder_type == 0 {
                    self.potgamefolder = path;
                } else if folder_type == 1 {
                    self.potsavefolder = path;
                }
                Task::none()
            }
            Message::ApplyFolder => {
                let mut change = false;
                if !self.potgamefolder.is_empty() {
                    change = true;
                    let contents =
                        fs::read_to_string(&self.cfg_path).expect("Could not grab contents");
                    let mut json: Config =
                        serde_json::from_str(&contents).expect("could not parse contents");
                    json.savedir = format!("{}", &self.potgamefolder);
                    fs::write(
                        &self.cfg_path,
                        serde_json::to_string_pretty(&json).expect("Couldn't translate config"),
                    )
                    .expect("Couldn't write to config");
                }
                if !self.potsavefolder.is_empty() {
                    change = true;
                    let contents =
                        fs::read_to_string(&self.cfg_path).expect("Could not grab contents");
                    let mut json: Config =
                        serde_json::from_str(&contents).expect("could not parse contents");
                    json.backupdir = format!("{}", &self.potsavefolder);
                    fs::write(
                        &self.cfg_path,
                        serde_json::to_string_pretty(&json).expect("Couldn't translate config"),
                    )
                    .expect("Couldn't write to config");
                }
                if change {
                    self.refresh_folders();
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }
    pub fn view(&self) -> Column<'_, Message> {
        if self.showsettings {
            self.settings_ui()
        } else {
            self.save_ui()
        }
    }

    fn refresh_folders(&mut self) {
        let config =
            fs::read_to_string(&self.cfg_path).expect("Config error: could not find config.json");
        let data: Config =
            serde_json::from_str(&config).expect("Config error: could not parse file");

        self.gamefolder = format!("{}", data.savedir);
        self.selected_disp = None;
        self.savefolder = format!("{}", data.backupdir);
        let saveops = savemanager::compile_saves(&self.gamefolder);
        self.selected_save.clear();
        self.selected_path = format!(
            "{}/{}",
            self.savefolder,
            self.selected_save.replace(".rsg", "")
        );

        self.save_slot_info = Vec::new();
        for save in &saveops {
            if fs::metadata(format!("{}/{}", &self.savefolder, save.replace(".rsg", ""))).is_ok() {
                let backups = savemanager::compile_saves(&format!(
                    "{}/{}",
                    &self.savefolder,
                    save.replace(".rsg", "")
                ));
                for backup in backups {
                    self.save_slot_info.push(savemanager::read_info(&format!(
                        "{}/{}{}",
                        &self.savefolder,
                        save.replace(".rsg", ""),
                        backup
                    )));
                }
            }
        }
        self.refresh();
    }

    fn open_folder(&self, folder_type: i32) -> Task<Message> {
        Task::future(
            rfd::AsyncFileDialog::new().pick_folder(), // <-- Launch the dialog window.
        )
        .then(move |handle| match handle {
            Some(folder_handle) => {
                let path = folder_handle.path().display().to_string();

                Task::done(Message::FolderSelected(folder_type, path))
            }
            None => Task::done(Message::FolderCanceled),
        })
    }

    fn settings_ui(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> = column!(
            create_img(&format!("Banner.png"), image::FilterMethod::Linear).width(Length::Fill),
        )
        .spacing(10);
        screening = screening.push(
            column![
                text!("Settings").width(Length::Fill).center().size(30),
                rule::horizontal(2),
                button(text!("Back").width(Length::Fill).center())
                    .on_press(Message::Settings)
                    .width(Length::Fill),
                row![
                    text!("Game Folder:").height(30).center(),
                    text_input(&self.gamefolder, &self.potgamefolder).size(15),
                    button(
                        create_img(&format!("folder.png"), image::FilterMethod::Linear).width(25)
                    )
                    .style(button::text)
                    .height(30)
                    .on_press(Message::OpenFolder(0)),
                    button("Apply").height(30).on_press(Message::ApplyFolder),
                ],
                row![
                    text!("Save Folder:").height(30).center(),
                    text_input(&self.savefolder, &self.potsavefolder).size(15),
                    button(
                        create_img(&format!("folder.png"), image::FilterMethod::Linear).width(25)
                    )
                    .style(button::text)
                    .height(30)
                    .on_press(Message::OpenFolder(1)),
                    button("Apply").height(30).on_press(Message::ApplyFolder),
                ]
                .spacing(5),
            ]
            .padding(5)
            .spacing(10),
        );

        screening
    }

    fn save_ui(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> = column!(
            create_img(&format!("Banner.png"), image::FilterMethod::Linear).width(Length::Fill),
            self.save_selector()
        );
        if self.selected_disp != None {
            screening = screening.push(self.save_info());
            screening = screening.push(self.save_slots());
        }

        screening
    }

    // You know who ELSE uses recursion? You know who ELSE uses recursion? You know who--
    // Should find an unused save number.
    fn findunusedsavename(&mut self, save_num: i64) {
        self.defaultsavename = format!("Save {}", save_num);
        if fs::exists(format!(
            "{}/{}.rsg",
            &self.selected_path, &self.defaultsavename
        ))
        .expect("Could not check if file exists!")
        {
            self.findunusedsavename(save_num + 1);
        }
    }

    fn refresh(&mut self) {
        self.saves = savemanager::compile_saves(&self.gamefolder);
        self.backups = savemanager::compile_saves(&self.selected_path);
        self.savedisp = savemanager::generate_save_display(&self.gamefolder, &self.saves);
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
            if fullpath.contains("Arena0") {
                self.gametype = format!("Arena");
            } else {
                self.gametype = format!("Campaign");
            }
            let save_info = savemanager::read_info(&fullpath);
            self.charname = format!("{}", save_info.name);
            self.charloc = format!("{}", save_info.location);
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
            row![
                text_input(&self.defaultsavename, &self.newsavename)
                    .width(Length::FillPortion(1))
                    .on_input(Message::ContentChanged),
                button(text!("Create New Save").center().width(Length::Fill))
                    .width(Length::FillPortion(2))
                    .on_press(Message::NewSave)
            ]
            .align_y(iced::Alignment::Center),
        ]
        .padding(5)
        .spacing(10);
        let mut scrollarea = column![].spacing(10);
        for save in &self.save_slot_info {
            if save.path.contains(&self.selected_path) {
                scrollarea = scrollarea.push(self.save_slot(&save));
            }
        }
        let scroll = scrollable(scrollarea);
        screening = screening.push(scroll);
        screening
    }

    fn save_slot(&self, info: &SaveInfo) -> Column<'_, Message> {
        let meta = fs::metadata(&info.path).unwrap();
        let time_saved: DateTime<Local> =
            meta.modified().expect("Couldn't find creation date").into();
        let file_size: f64 = (meta.size() as f64) / 1000000.0;
        column![
            container(
                row!(
                    column!(
                        text!(
                            "{}",
                            info.path
                                .replace(&self.selected_path, "")
                                .replace(".rsg", "")
                                .replace("/", "")
                        )
                        .size(20),
                        text!("{:.2} MB", file_size)
                            .size(15)
                            .color(color!(150, 150, 150))
                    )
                    .width(220),
                    rule::vertical(2),
                    column![
                        text!("Name: {}", &info.name)
                            .height(20)
                            .center()
                            .width(Length::Fill),
                        rule::horizontal(2),
                        text!("Location: {}", &info.location)
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
                    button(column![
                        create_img(&format!("save.png"), image::FilterMethod::Linear)
                            .height(Length::FillPortion(3)),
                        text!("Save")
                            .height(Length::FillPortion(1))
                            .width(Length::Fill)
                            .center()
                    ])
                    .on_press(Message::OverwriteSave(
                        self.save_slot_info
                            .iter()
                            .position(|r| r.path == info.path)
                            .unwrap()
                    ))
                    .width(75)
                    .height(75),
                    button(column![
                        create_img(&format!("restore.png"), image::FilterMethod::Linear)
                            .height(Length::FillPortion(3)),
                        text!("Restore")
                            .height(Length::FillPortion(1))
                            .width(Length::Fill)
                            .center()
                    ])
                    .on_press(Message::RestoreSave(
                        self.save_slot_info
                            .iter()
                            .position(|r| r.path == info.path)
                            .unwrap()
                    ))
                    .width(75)
                    .height(75),
                    button(column![
                        create_img(&format!("delete.png"), image::FilterMethod::Linear)
                            .height(Length::FillPortion(3)),
                        text!("Delete")
                            .height(Length::FillPortion(1))
                            .width(Length::Fill)
                            .center()
                    ])
                    .on_press(Message::DeleteSave(
                        self.save_slot_info
                            .iter()
                            .position(|r| r.path == info.path)
                            .unwrap()
                    ))
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
