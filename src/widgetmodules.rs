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
use rfd;
use std::env;
use std::f64;
use std::fs;
use std::io::Error;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

fn create_img<'a>(
    path: &String,
    filter_method: image::FilterMethod,
) -> Result<Container<'a, Message>, Error> {
    let banpath: String = format!("{}/images/{}", env::current_dir()?.display(), path);
    Ok(center_x(image(banpath).filter_method(filter_method)))
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

        let gamedir = format!("{}", data.gamedir);
        let savedir = format!("{}", data.savedir);
        let saveops: Vec<String>;
        if fs::metadata(&gamedir).is_ok() {
            saveops = savemanager::compile_saves(&gamedir).unwrap_or(Vec::new());
        } else {
            saveops = Vec::new();
        }

        let mut saveinfo = Vec::new();
        for save in &saveops {
            if fs::metadata(format!("{}/{}", &savedir, save.replace(".rsg", ""))).is_ok() {
                let mut backups: Vec<String> = Vec::new();
                match savemanager::compile_saves(&format!(
                    "{}/{}",
                    &savedir,
                    save.replace(".rsg", "")
                )) {
                    Ok(data) => {
                        backups = data;
                    }
                    Err(_) => {
                        // TODO: add better error tracker here
                        println!(
                            "ERROR: Could not compile backup saves. Continuing with empty save array."
                        );
                    }
                }
                for backup in backups {
                    match savemanager::read_info(&format!(
                        "{}/{}/{}",
                        &savedir,
                        save.replace(".rsg", ""),
                        backup
                    )) {
                        Ok(data) => saveinfo.push(data),
                        Err(_) => {
                            println!(
                                "ERROR: Could not compile information on save {}/{}/{}. Skipping.",
                                &savedir,
                                save.replace(".rsg", ""),
                                backup
                            );
                        }
                    };
                }
            }
        }
        let mut displaynames: Vec<String> = Vec::new();
        match savemanager::generate_save_display(&gamedir, &saveops) {
            Ok(data) => {
                displaynames = data;
            }
            Err(_) => {
                println!(
                    "ERROR: Could not generate a save display for the game saves. Continuing with an empty array."
                );
            }
        }

        Self {
            cfg_path: cfgpath,
            gamefolder: gamedir.clone(),
            savefolder: savedir.clone(),

            selected_disp: None,
            selected_save: String::new(),
            savedisp: displaynames,
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
                let mut error = false;
                self.selected_disp = Some(option);
                if self.selected_disp != None {
                    self.selected_save = self.saves[self
                        .savedisp
                        .iter()
                        .position(|disp| disp == &self.selected_disp.clone().unwrap())
                        .unwrap()]
                    .clone();
                    self.update_info();
                    self.selected_path = format!(
                        "{}/{}",
                        self.savefolder,
                        self.selected_save.replace(".rsg", "")
                    );
                    if fs::metadata(&self.selected_path).is_ok() {
                        match savemanager::compile_saves(&self.selected_path) {
                            Ok(saves) => {
                                self.backups = saves;
                            }
                            Err(_) => {
                                error = true;
                            }
                        };
                    } else {
                        self.backups = Vec::new();
                    };
                } else {
                    self.selected_save = String::new();
                }
                self.findunusedsavename(self.backups.len() as i64 + 1);
                if !error {
                    Task::none()
                } else {
                    self.throw_err("Could not compile saves!".to_string())
                }
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
                match savemanager::new_save(
                    &format!("{}/{}", self.gamefolder, self.selected_save),
                    &self.selected_path,
                    savename,
                ) {
                    Ok(_) => {
                        let mut error = false;
                        let mut existent = false;
                        for saveinfo in &self.save_slot_info {
                            if saveinfo.path == format!("{}/{}.rsg", &self.selected_path, savename)
                            {
                                existent = true;
                                let pos = self
                                    .save_slot_info
                                    .iter()
                                    .position(|r| r.path == saveinfo.path)
                                    .unwrap();
                                match savemanager::read_info(&format!(
                                    "{}/{}.rsg",
                                    &self.selected_path, savename
                                )) {
                                    Ok(info) => {
                                        self.save_slot_info[pos] = info;
                                    }
                                    Err(_) => {
                                        error = true;
                                    }
                                };
                                break;
                            }
                        }
                        if !existent {
                            match savemanager::read_info(&format!(
                                "{}/{}.rsg",
                                &self.selected_path, savename
                            )) {
                                Ok(info) => {
                                    self.save_slot_info.push(info);
                                }
                                Err(_) => {
                                    error = true;
                                }
                            };
                        }
                        if error {
                            self.throw_err("Could not save the game!".to_string())
                        } else {
                            self.newsavename.clear();
                            match savemanager::compile_saves(&self.selected_path) {
                                Ok(data) => {
                                    self.backups = data;
                                    self.findunusedsavename(self.backups.len() as i64 + 1);
                                    Task::none()
                                }
                                Err(_) => self.throw_err(
                                    "Saved the game, but could not update the save display!"
                                        .to_string(),
                                ),
                            }
                        }
                    }
                    Err(_) => self.throw_err("Could not save the game!".to_string()),
                }
            }
            Message::OverwriteSave(tar) => {
                match savemanager::copy_save(
                    &format!("{}/{}", self.gamefolder, self.selected_save),
                    &self.save_slot_info[tar].path,
                ) {
                    Ok(_) => {
                        self.refresh();
                        Task::none()
                    }
                    Err(_) => self.throw_err("Could not overwrite the save!".to_string()),
                }
            }
            Message::RestoreSave(tar) => {
                match savemanager::copy_save(
                    &self.save_slot_info[tar].path,
                    &format!("{}/{}", self.gamefolder, self.selected_save),
                ) {
                    Ok(_) => {
                        self.update_info();
                        self.refresh();
                        Task::none()
                    }
                    Err(_) => self.throw_err("Could not restore the save!".to_string()),
                }
            }
            Message::DeleteSave(tar) => match fs::remove_file(&self.save_slot_info[tar].path) {
                Ok(_) => {
                    self.save_slot_info.remove(tar);
                    self.refresh();
                    self.findunusedsavename(self.backups.len() as i64 + 1);
                    Task::none()
                }
                Err(_) => self.throw_err("Could not delete the save!".to_string()),
            },
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
                let mut error = 0;
                if !self.potgamefolder.is_empty() {
                    match fs::read_to_string(&self.cfg_path) {
                        Ok(data) => {
                            let contents = data;
                            match serde_json::from_str(&contents) {
                                Ok(parsed) => {
                                    let mut json: Config = parsed;
                                    json.savedir = format!("{}", &self.potgamefolder);
                                    match serde_json::to_string_pretty(&json) {
                                        Ok(data) => match fs::write(&self.cfg_path, data) {
                                            Ok(_) => {
                                                change = true;
                                            }
                                            Err(_) => {
                                                error = 4;
                                            }
                                        },
                                        Err(_) => {
                                            error = 3;
                                        }
                                    }
                                }
                                Err(_) => {
                                    error = 2;
                                }
                            }
                        }
                        Err(_) => {
                            error = 1;
                        }
                    }
                }
                if !self.potsavefolder.is_empty() {
                    match fs::read_to_string(&self.cfg_path) {
                        Ok(data) => {
                            let contents = data;
                            match serde_json::from_str(&contents) {
                                Ok(parsed) => {
                                    let mut json: Config = parsed;
                                    json.backupdir = format!("{}", &self.potsavefolder);
                                    match serde_json::to_string_pretty(&json) {
                                        Ok(data) => match fs::write(&self.cfg_path, data) {
                                            Ok(_) => {
                                                change = true;
                                            }
                                            Err(_) => {
                                                error = 4;
                                            }
                                        },
                                        Err(_) => {
                                            error = 3;
                                        }
                                    }
                                }
                                Err(_) => {
                                    error = 2;
                                }
                            }
                        }
                        Err(_) => {
                            error = 1;
                        }
                    }
                }
                if change {
                    match self.refresh_folders() {
                        Ok(_) => {}
                        Err(_) => {
                            error = 5;
                        }
                    };
                }
                match error {
                    0 => Task::none(),
                    1 => self.throw_err("Could not read the config file!".to_string()),
                    2 => self.throw_err("Could not parse config data!".to_string()),
                    3 => self.throw_err("Could not serialize new config data!".to_string()),
                    4 => self.throw_err("Could not write to the config file!".to_string()),
                    5 => self.throw_err("Could not refresh folders!".to_string()),
                    _ => Task::none(),
                }
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

    fn throw_err(&self, message: String) -> Task<Message> {
        Task::perform(
            async move {
                rfd::AsyncMessageDialog::new()
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error")
                    .set_description(message)
                    .show()
                    .await
            },
            |_| Message::MessageClosed,
        )
    }

    fn refresh_folders(&mut self) -> Result<(), Error> {
        let config =
            fs::read_to_string(&self.cfg_path).expect("Config error: could not find config.json");
        let data: Config =
            serde_json::from_str(&config).expect("Config error: could not parse file");

        self.gamefolder = format!("{}", data.savedir);
        self.selected_disp = None;
        self.savefolder = format!("{}", data.backupdir);
        let mut saveops: Vec<String> = Vec::new();
        if fs::metadata(&self.gamefolder).is_ok() {
            match savemanager::compile_saves(&self.gamefolder) {
                Ok(saves) => {
                    saveops = saves;
                }
                Err(_) => {}
            }
        }
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
                ))
                .unwrap_or(Vec::new());
                for backup in backups {
                    match savemanager::read_info(&format!(
                        "{}/{}/{}",
                        &self.savefolder,
                        save.replace(".rsg", ""),
                        backup
                    )) {
                        Ok(savedata) => self.save_slot_info.push(savedata),
                        Err(_) => {}
                    }
                }
            }
        }
        self.refresh();
        Ok(())
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
        let mut screening: Column<'_, Message>;
        match create_img(&format!("Banner.png"), image::FilterMethod::Linear) {
            Ok(image) => {
                screening = column!(image.width(Length::Fill));
            }
            Err(_) => {
                screening = column!(text!("404").center().width(Length::Fill));
            }
        }
        screening = screening.push(
            column![
                button(text!("Back").width(Length::Fill).center())
                    .on_press(Message::Settings)
                    .width(Length::Fill),
                row![
                    text!("Game Folder:")
                        .height(30)
                        .width(Length::FillPortion(2))
                        .center(),
                    text_input(&self.gamefolder, &self.potgamefolder)
                        .size(15)
                        .width(Length::FillPortion(15)),
                    match create_img(&format!("folder.png"), image::FilterMethod::Linear) {
                        Ok(image) => {
                            button(image.width(25))
                                .width(Length::FillPortion(1))
                                .style(button::text)
                                .height(30)
                                .on_press(Message::OpenFolder(0))
                        }
                        Err(_) => {
                            button(text!("404").width(25))
                                .width(Length::FillPortion(1))
                                .style(button::text)
                                .height(30)
                                .on_press(Message::OpenFolder(0))
                        }
                    },
                    button(text!("Apply").width(Length::FillPortion(1)).center())
                        .height(30)
                        .on_press(Message::ApplyFolder)
                        .width(Length::FillPortion(1)),
                ],
                if !fs::metadata(&self.gamefolder).is_ok() {
                    self.gen_error(&"This game folder is invalid!".to_string())
                } else {
                    column!()
                },
                row![
                    text!("Save Folder:")
                        .height(30)
                        .width(Length::FillPortion(2))
                        .center(),
                    text_input(&self.savefolder, &self.potsavefolder)
                        .size(15)
                        .width(Length::FillPortion(15)),
                    match create_img(&format!("folder.png"), image::FilterMethod::Linear) {
                        Ok(image) => {
                            button(image.width(25))
                                .width(Length::FillPortion(1))
                                .style(button::text)
                                .height(30)
                                .on_press(Message::OpenFolder(1))
                        }
                        Err(_) => {
                            button(text!("404").width(25))
                                .width(Length::FillPortion(1))
                                .style(button::text)
                                .height(30)
                                .on_press(Message::OpenFolder(1))
                        }
                    },
                    button(text!("Apply").width(Length::FillPortion(1)).center())
                        .height(30)
                        .on_press(Message::ApplyFolder)
                        .width(Length::FillPortion(1)),
                ],
                if !fs::metadata(&self.savefolder).is_ok() {
                    self.gen_error(&"This save folder is invalid!".to_string())
                } else {
                    column!()
                },
            ]
            .padding(5)
            .spacing(10),
        );

        screening
    }

    fn save_ui(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message>;
        match create_img(&format!("Banner.png"), image::FilterMethod::Linear) {
            Ok(image) => {
                screening = column!(image.width(Length::Fill), self.save_selector());
            }
            Err(_) => {
                screening = column!(
                    text!("404").center().width(Length::Fill),
                    self.save_selector()
                );
            }
        }
        if !fs::metadata(&self.savefolder).is_ok() || !fs::metadata(&self.gamefolder).is_ok() {
            if !fs::metadata(&self.gamefolder).is_ok() {
                screening = screening.push(
                    self.gen_error(
                        &"The current game folder is invalid! Resolve this issue in the Settings."
                            .to_string(),
                    ),
                );
            }
            if !fs::metadata(&self.savefolder).is_ok() {
                screening = screening.push(
                    self.gen_error(
                        &"The current save folder is invalid! Resolve this issue in the Settings."
                            .to_string(),
                    ),
                );
            }
        } else {
            if self.selected_disp != None {
                screening = screening.push(self.save_info());
                screening = screening.push(self.save_slots());
            }
        }

        screening
    }

    fn gen_error(&self, message: &String) -> Column<'_, Message> {
        column!(text!("{}", message).color(color!(255, 0, 0)))
            .width(Length::Fill)
            .padding(5)
    }

    // Finds an unused save name.
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
        if fs::metadata(&self.gamefolder).is_ok() {
            self.saves = savemanager::compile_saves(&self.gamefolder).unwrap_or(Vec::new());
            self.savedisp =
                savemanager::generate_save_display(&self.gamefolder, &self.saves).expect("Broken");
        }
        if fs::metadata(&self.selected_path).is_ok() {
            self.backups = savemanager::compile_saves(&self.selected_path).unwrap_or(Vec::new());
            // Check if there are new saves here, then compile information about them into the cache
        }
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

    fn update_info(&mut self) {
        if self.selected_disp != None {
            let fullpath: String = format!("{}/{}", &self.gamefolder, &self.selected_save);
            let metadata = fs::metadata(&fullpath).unwrap();
            match metadata.created() {
                Ok(date) => {
                    let time: DateTime<Local> = date.into();
                    self.created_date = format!("{}", time.format("%m/%d/%Y at %-I:%M %p"));
                }
                Err(_) => {
                    self.created_date = "Unknown Time".to_string();
                }
            }
            if fullpath.contains("Arena0") {
                self.gametype = format!("Arena");
            } else {
                self.gametype = format!("Campaign");
            }
            let save_info = savemanager::read_info(&fullpath).expect("broken 5");
            self.charname = format!("{}", save_info.name);
            self.charloc = format!("{}", save_info.location);
        }
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
        let time_str: String;
        match meta.modified() {
            Ok(date) => {
                let time: DateTime<Local> = date.into();
                time_str = format!("{}", time.format("%m/%d/%Y at %-I:%M %p"));
            }
            Err(_) => {
                time_str = "Unknown Time".to_string();
            }
        };
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
                    text!("Saved {}", time_str)
                        .height(75)
                        .center()
                        .width(Length::Fill),
                    rule::vertical(2),
                    button(column![
                        match create_img(&format!("save.png"), image::FilterMethod::Linear) {
                            Ok(img) => {
                                container(img.height(Length::FillPortion(3)))
                            }
                            Err(_) => {
                                container(text!("404"))
                            }
                        },
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
                        match create_img(&format!("restore.png"), image::FilterMethod::Linear) {
                            Ok(img) => {
                                container(img.height(Length::FillPortion(3)))
                            }
                            Err(_) => {
                                container(text!("404").height(Length::FillPortion(3)).center())
                            }
                        },
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
                        match create_img(&format!("delete.png"), image::FilterMethod::Linear) {
                            Ok(img) => {
                                container(img.height(Length::FillPortion(3)))
                            }
                            Err(_) => {
                                container(text!("404").height(Length::FillPortion(3)).center())
                            }
                        },
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
