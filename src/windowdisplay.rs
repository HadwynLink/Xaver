use crate::{messages::*, savemanager::*, utils::*};
use iced::{
    Length, Task, color,
    widget::{
        Column, Row, button, column, container, image, pick_list, row, rule, scrollable, text,
        text_input,
    },
};
use std::{fs, io::ErrorKind, process::Command};

pub struct Xaver {
    // Tracks what panel to display
    // 0 = Main; 1 = Settings; 2 = Setup
    window_state: i32,

    // Variables to keep track of certain file locations
    game_folder: String, // Location of game folder (game saves)
    save_folder: String, // Location of save folder (backup saves)

    // Variables to keep track of saves
    game_saves: Vec<SaveInfo>, // List of all game saves in the game folder and related information
    selected_save: Option<SaveInfo>, // Selected game save and related information
    selected_save_path: String, // Path that saves of that directory should be in
    backup_saves: Vec<SaveInfo>, // List of all backup saves in the save folder and their information

    // Variables to create a new save
    default_save_name: String, // Auto-generated name for a new save (Save '#' format)
    new_save_name: String,     // User-input name for a new save

    // Variables to track user-input directories for game/save folders
    game_folder_input: String, // User-input directory for game folder
    save_folder_input: String, // User-input directory for save folder
}

impl Xaver {
    /* ------------------ CORE FUNCTIONS ------------------ */
    // Default initialization of the structure
    pub fn default() -> Self {
        // Variable staging
        let s_window_state: i32;
        let mut s_game_folder: String;
        let mut s_save_folder: String;
        let s_game_saves: Vec<SaveInfo>;
        let mut s_backup_saves: Vec<SaveInfo> = Vec::new();

        // Gets the config and extracts data from it
        let data = get_config();
        match data {
            Ok(contents) => {
                s_window_state = 0;
                s_game_folder = contents.gamedir;
                s_save_folder = contents.savedir;
                if fs::metadata(&s_game_folder).is_ok() {
                    s_game_saves = compile_info(&s_game_folder).unwrap_or(Vec::new());
                } else {
                    s_game_saves = Vec::new();
                }
                for save in &s_game_saves {
                    if let Some((_, fname)) = save.path.replace("\\", "/").rsplit_once('/') {
                        if fs::metadata(format!("{}/{}", &s_save_folder, fname.replace(".rsg", "")))
                            .is_ok()
                        {
                            s_backup_saves.append(
                                &mut compile_info(&format!(
                                    "{}/{}",
                                    &s_save_folder,
                                    fname.replace(".rsg", "")
                                ))
                                .unwrap_or(Vec::new()),
                            );
                        }
                    }
                }
            }
            Err(error) => {
                s_window_state = match error.kind() {
                    ErrorKind::NotFound => 2,
                    _ => 0,
                };
                s_game_folder = String::new();
                s_save_folder = String::new();
                s_game_saves = Vec::new();
                s_backup_saves = Vec::new();
            }
        }
        if s_window_state == 2 {
            s_game_folder = recommend_game_folder();
            s_save_folder = recommend_backup_folder();
        }
        Self {
            // Tracks what panel to display
            // 0 = Main Page; 1 = Settings
            window_state: s_window_state,

            // Variables to keep track of certain file locations
            game_folder: s_game_folder,
            save_folder: s_save_folder,

            // Variables to keep track of saves
            game_saves: s_game_saves,
            selected_save: None,
            selected_save_path: String::new(),
            backup_saves: s_backup_saves,

            // Variables to create a new save
            default_save_name: String::new(),
            new_save_name: String::new(),

            // Variables to track user-input directories for game/save folders
            game_folder_input: String::new(),
            save_folder_input: String::new(),
        }
    }
    // Handles user input
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                self.refresh();
                Task::none()
            }
            Message::Launch => {
                let _ = Command::new("xdg-open").arg("steam://run/362490").spawn();
                Task::none()
            }
            Message::SwitchDisplay(state_num) => {
                if self.window_state == 2 && state_num != 2 {
                    self.save_folder_input.clear();
                    self.game_folder_input.clear();
                }
                self.window_state = state_num;
                if self.window_state == 2 {
                    self.setup_folder_suggestions();
                }
                Task::none()
            }
            Message::SaveSelected(option) => {
                self.selected_save = Some(option);
                if let Some(save_info) = self.selected_save.as_ref() {
                    if let Some((_, fname)) = save_info.path.replace("\\", "/").rsplit_once('/') {
                        self.selected_save_path =
                            format!("{}/{}", &self.save_folder, fname.replace(".rsg", ""));
                    }
                }
                self.find_unused_name(find_num_saves(&self.backup_saves, &self.selected_save_path));
                Task::none()
            }
            Message::SaveNameChanged(save_name) => {
                self.new_save_name = save_name;
                Task::none()
            }
            Message::NewSave => {
                let save_name = if self.new_save_name != "" {
                    &self.new_save_name
                } else {
                    &self.default_save_name
                };
                if let Some(save_info) = &self.selected_save {
                    match new_save(&save_info.path.replace("\\", "/"), &self.selected_save_path, save_name) {
                        Ok(_) => {
                            // TODO: Signal success of the operation to user
                            match read_info(&format!(
                                "{}/{}.rsg",
                                &self.selected_save_path, save_name
                            )) {
                                Ok(new_save_info) => {
                                    if !self.backup_saves.contains(&new_save_info) {
                                        self.backup_saves.push(new_save_info);
                                        self.find_unused_name(find_num_saves(
                                            &self.backup_saves,
                                            &self.selected_save_path,
                                        ));
                                    } else {
                                        if let Some(pos) = self
                                            .backup_saves
                                            .iter()
                                            .position(|x| x == &new_save_info)
                                        {
                                            self.backup_saves[pos] = new_save_info;
                                        }
                                    }
                                    self.new_save_name.clear();
                                    Task::none()
                                }
                                Err(err_type) => self.error_popup(format!(
                                    "Error: Could not update the save display! Error Code:\n{:#?}",
                                    err_type
                                )),
                            }
                        }
                        Err(err_type) => self.error_popup(format!(
                            "Error: Could not create new save! Error Code:\n{:#?}",
                            err_type
                        )),
                    }
                } else {
                    self.error_popup(format!(
                        "Error: Could not create new save! Could not parse the information of the save to back up."
                    )) // I can't think of any circumstance where this would happen, but just in case there's an error message
                }
            }
            Message::OverwriteSave(tar) => {
                if let Some(save_info) = &self.selected_save {
                    match copy_save(&save_info.path.replace("\\", "/"), &tar) {
                        Ok(_) => match read_info(&tar) {
                            Ok(new_save_info) => {
                                if !self.backup_saves.contains(&new_save_info) {
                                    self.backup_saves.push(new_save_info);
                                    self.find_unused_name(find_num_saves(
                                        &self.backup_saves,
                                        &self.selected_save_path,
                                    ));
                                } else {
                                    if let Some(pos) =
                                        self.backup_saves.iter().position(|x| x == &new_save_info)
                                    {
                                        self.backup_saves[pos] = new_save_info;
                                    }
                                }
                                Task::none()
                            }
                            Err(err_type) => self.error_popup(format!(
                                "Error: Could not update the save display! Error Code:\n{:#?}",
                                err_type
                            )),
                        },
                        Err(err_type) => self.error_popup(format!(
                            "Error: Could not overwrite the save! Error Code:\n{:#?}",
                            err_type
                        )),
                    }
                } else {
                    self.error_popup(format!(
                            "Error: Could not overwrite the save! Could not parse the information of the save to overwrite."
                        )) // I can't think of any circumstance where this would happen, but just in case there's an error message
                }
            }
            Message::RestoreSave(tar) => {
                if let Some(save_info) = &self.selected_save {
                    match copy_save(&tar, &save_info.path.replace("\\", "/")) {
                        Ok(_) => match read_info(&save_info.path.replace("\\", "/")) {
                            Ok(restored_save_info) => {
                                if let Some(pos) = self
                                    .game_saves
                                    .iter()
                                    .position(|x| x == &restored_save_info)
                                {
                                    self.game_saves[pos] = restored_save_info;
                                    self.selected_save = Some(self.game_saves[pos].clone());
                                }
                                Task::none()
                            }
                            Err(err_type) => self.error_popup(format!(
                                "Error: Could not update the save display! Error Code:\n{:#?}",
                                err_type
                            )),
                        },
                        Err(err_type) => self.error_popup(format!(
                            "Error: Could not restore the save! Error Code:\n{:#?}",
                            err_type
                        )),
                    }
                } else {
                    self.error_popup(format!(
                            "Error: Could not restore the save! Could not parse the information of the save to restore."
                        )) // This error shouldn't happen but the case is here anyway
                }
            }
            Message::DeleteSave(tar) => match fs::remove_file(&tar) {
                Ok(_) => {
                    if let Some(pos) = self.backup_saves.iter().position(|x| x.path == tar) {
                        self.backup_saves.remove(pos);
                        self.find_unused_name(find_num_saves(
                            &self.backup_saves,
                            &self.selected_save_path,
                        ));
                        Task::none()
                    } else {
                        self.error_popup(
                            "Error: Deleted save, but could not update save list!".to_string(),
                        ) // This shouldn't be possible
                    }
                }
                Err(err_type) => self.error_popup(format!(
                    "Could not delete the save! Error Code:\n{:#?}",
                    err_type
                )),
            },
            Message::GameFolderChanged(folder_name) => {
                self.game_folder_input = folder_name;
                Task::none()
            }
            Message::SaveFolderChanged(folder_name) => {
                self.save_folder_input = folder_name;
                Task::none()
            }
            Message::OpenFolder(foldertype) => self.open_folder(foldertype),
            Message::FolderSelected(folder_type, path) => {
                if folder_type == 0 {
                    self.game_folder_input = path;
                } else if folder_type == 1 {
                    self.save_folder_input = path;
                }
                Task::none()
            }
            Message::ApplyFolder => {
                let mut change = false;
                match get_config() {
                    Ok(mut cfg_info) => {
                        if !self.game_folder_input.is_empty() {
                            cfg_info.gamedir = format!("{}", &self.game_folder_input).replace("\\", "/");
                            change = true;
                            self.game_folder_input.clear();
                        }
                        if !self.save_folder_input.is_empty() {
                            cfg_info.savedir = format!("{}", &self.save_folder_input).replace("\\", "/");
                            change = true;
                            self.save_folder_input.clear();
                        }
                        match save_config(cfg_info) {
                            Ok(_) => {
                                if change == true {
                                    self.refresh();
                                }
                                Task::none()
                            }
                            Err(err_type) => self.error_popup(format!(
                                "Error: Could not change the config! Error Code: {}",
                                err_type
                            )),
                        }
                    }
                    Err(err_type) => self.error_popup(format!(
                        "Error: Could not read the config! Error Code: {}",
                        err_type
                    )),
                }
            }
            Message::NewConfig => match new_config(Config {
                gamedir: if self.game_folder_input.is_empty() {
                    self.game_folder.clone().replace("\\", "/")
                } else {
                    self.game_folder_input.clone().replace("\\", "/")
                },
                savedir: if self.save_folder_input.is_empty() {
                    self.save_folder.clone().replace("\\", "/")
                } else {
                    self.save_folder_input.clone().replace("\\", "/")
                },
            }) {
                Ok(_) => {
                    self.refresh();
                    match create_save_folder(&self.save_folder) {
                        Ok(_) => Task::none(),
                        Err(err_type) => self.error_popup(format!(
                            "Error: Could not create or verify the specified backup folder! Error Code:\n{:#?}",
                            err_type
                        )),
                    }
                }
                Err(err_type) => self.error_popup(format!(
                    "Error: Could not create a new config file! Error Code:\n{:#?}",
                    err_type
                )),
            },
            _ => Task::none(),
        }
    }
    // Graphical output of the struct
    pub fn view(&self) -> Column<'_, Message> {
        match self.window_state {
            0 => self.save_ui(),
            1 => self.settings_ui(),
            2 => self.setup_ui(),
            _ => self.save_ui(),
        }
    }

    /* ------------------ DISPLAY HOOKS ------------------ */
    // Handles the saving and loading interface
    fn save_ui(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> = column!(
            create_img(&format!("Banner.png"), image::FilterMethod::Linear).width(Length::Fill),
            self.save_selector()
        );
        if !fs::metadata(&self.save_folder).is_ok() || !fs::metadata(&self.game_folder).is_ok() {
            if !fs::metadata(&self.game_folder).is_ok() {
                screening = screening.push(
                    self.throw_error(
                        &"The current game folder is invalid! Resolve this issue in the Settings."
                            .to_string(),
                    ),
                );
            }
            if !fs::metadata(&self.save_folder).is_ok() {
                screening = screening.push(
                    self.throw_error(
                        &"The current save folder is invalid! Resolve this issue in the Settings."
                            .to_string(),
                    ),
                );
            }
        } else {
            if self.selected_save != None {
                screening = screening.push(self.save_info());
                screening = screening.push(self.save_slot_tab());
            }
        }

        screening
    }
    // Display logic for save selection -- Hooked into save_ui
    fn save_selector(&self) -> Column<'_, Message> {
        let screening = column![
            row![
                button(text!("Refresh").width(Length::Fill).center())
                    .on_press(Message::Refresh)
                    .width(Length::Fill),
                button(text!("Launch").width(Length::Fill).center())
                    .on_press(Message::Launch)
                    .width(Length::Fill),
                button(text!("Settings").width(Length::Fill).center())
                    .on_press(Message::SwitchDisplay(1))
                    .width(Length::Fill),
            ]
            .spacing(5),
            row!(
                text("Current Save:").height(32).center(),
                pick_list(
                    self.game_saves.clone(),
                    self.selected_save.clone(),
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
    // Display logic for detailed information about selected game save
    pub fn save_info(&self) -> Row<'_, Message> {
        if let Some(char_info) = self.selected_save.as_ref() {
            let screening = row![
                column![
                    text!(
                        "Save Type: {}",
                        match char_info.save_type {
                            0 => "Campaign",
                            1 => "Arena",
                            _ => "Unknown",
                        }
                    ),
                    text!("Character Name: {}", char_info.name),
                    text!("Location: {}", char_info.level.replace("\n", " "))
                ]
                .spacing(10)
                .width(Length::Fill),
                column![
                    text!(
                        "Created: {}",
                        char_info.created_date.format("%m/%d/%Y at %-I:%M %p")
                    ),
                    text!(
                        "Last modified: {}",
                        char_info.modified_date.format("%m/%d/%Y at %-I:%M %p")
                    ),
                    text!("File Size: {:.2} MB", char_info.file_size)
                ]
                .spacing(10)
                .width(Length::Fill)
            ]
            .padding(5)
            .spacing(5);
            screening
        } else {
            row!(text!("Could not compile save data!"))
        }
    }
    // Display logic for the list of backup saves
    fn save_slot_tab(&self) -> Column<'_, Message> {
        let mut screening = column![
            text!("Saves").size(25).width(Length::Fill).center(),
            rule::horizontal(2),
            row![
                text_input(&self.default_save_name, &self.new_save_name)
                    .width(Length::FillPortion(1))
                    .on_input(Message::SaveNameChanged),
                button(text!("Create New Save").center().width(Length::Fill))
                    .width(Length::FillPortion(2))
                    .on_press(Message::NewSave)
            ]
            .align_y(iced::Alignment::Center),
        ]
        .padding(5)
        .spacing(10);
        let mut scrollarea = column![].spacing(10);
        for save in &self.backup_saves {
            if save.path.replace("\\", "/").contains(&self.selected_save_path) {
                scrollarea = scrollarea.push(self.save_slot(&save));
            }
        }
        let scroll = scrollable(scrollarea);
        screening = screening.push(scroll);
        screening
    }
    // Container for a save slot
    fn save_slot(&self, info: &SaveInfo) -> Column<'_, Message> {
        column![
            container(
                row!(
                    column!(
                        text!(
                            "{}",
                            info.path.replace("\\", "/")
                                .replace(&self.selected_save_path, "")
                                .replace(".rsg", "")
                                .replace("/", "")
                        )
                        .size(20),
                        text!("{:.2} MB", info.file_size)
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
                        text!("Location: {}", &info.level)
                            .height(Length::Fill)
                            .center()
                            .width(Length::Fill)
                    ]
                    .spacing(5),
                    rule::vertical(2),
                    text!(
                        "Saved {}",
                        info.modified_date.format("%m/%d/%Y at %-I:%M %p")
                    )
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
                    .on_press(Message::OverwriteSave(info.path.clone()))
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
                    .on_press(Message::RestoreSave(info.path.clone()))
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
                    .on_press(Message::DeleteSave(info.path.clone()))
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

    // Handles the settings interface
    fn settings_ui(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> = column!(
            create_img(&format!("Banner.png"), image::FilterMethod::Linear).width(Length::Fill)
        );
        screening = screening.push(
            column![
                row!(
                    button(text!("Refresh").width(Length::Fill).center())
                        .on_press(Message::Refresh)
                        .width(Length::Fill),
                    button(text!("Back").width(Length::Fill).center())
                        .on_press(Message::SwitchDisplay(0))
                        .width(Length::Fill)
                )
                .spacing(5),
                row![
                    text!("Game Folder:")
                        .height(30)
                        .width(Length::FillPortion(2))
                        .center(),
                    text_input(&self.game_folder, &self.game_folder_input)
                        .size(15)
                        .on_input(Message::GameFolderChanged)
                        .width(Length::FillPortion(15)),
                    button(
                        create_img(&format!("folder.png"), image::FilterMethod::Linear).width(25)
                    )
                    .width(Length::FillPortion(1))
                    .style(button::text)
                    .height(30)
                    .on_press(Message::OpenFolder(0)),
                    button(text!("Apply").width(Length::FillPortion(1)).center())
                        .height(30)
                        .on_press(Message::ApplyFolder)
                        .width(Length::FillPortion(1)),
                ],
                if !fs::metadata(&self.game_folder).is_ok() {
                    self.throw_error(&"This game folder is invalid!".to_string())
                } else {
                    column!()
                },
                row![
                    text!("Save Folder:")
                        .height(30)
                        .width(Length::FillPortion(2))
                        .center(),
                    text_input(&self.save_folder, &self.save_folder_input)
                        .size(15)
                        .on_input(Message::SaveFolderChanged)
                        .width(Length::FillPortion(15)),
                    button(
                        create_img(&format!("folder.png"), image::FilterMethod::Linear).width(25)
                    )
                    .width(Length::FillPortion(1))
                    .style(button::text)
                    .height(30)
                    .on_press(Message::OpenFolder(1)),
                    button(text!("Apply").width(Length::FillPortion(1)).center())
                        .height(30)
                        .on_press(Message::ApplyFolder)
                        .width(Length::FillPortion(1)),
                ],
                if !fs::metadata(&self.save_folder).is_ok() {
                    self.throw_error(&"This save folder is invalid!".to_string())
                } else {
                    column!()
                },
            ]
            .padding(5)
            .spacing(10),
        );

        screening
    }

    // Handles the setup interface
    fn setup_ui(&self) -> Column<'_, Message> {
        let mut screening: Column<'_, Message> = column!(
            create_img(&format!("Banner.png"), image::FilterMethod::Linear).width(Length::Fill)
        );
        screening = screening.push(
            column!(
                text!("SETUP").width(Length::Fill).size(25).center(),
                rule::horizontal(2),
                text!("This is either your first time running Xaver, or the config file has been misplaced.")
                    .width(Length::Fill)
                    .center(),
                text!("Ensure that the following file locations are correct, then press Create New Config.")
                    .width(Length::Fill)
                    .center(),
                rule::horizontal(2),
                column![
                        text!("Game Folder: This is where the game stores save data. It should have .rsg files in it.")
                                .height(30),
                        row![
                                text_input(&self.game_folder, &self.game_folder_input)
                                        .size(15)
                                        .on_input(Message::GameFolderChanged)
                                        .width(Length::FillPortion(15)),
                                button(
                                        create_img(&format!("folder.png"), image::FilterMethod::Linear).width(25)
                                )
                                .width(Length::FillPortion(1))
                                .style(button::text)
                                .height(30)
                                .on_press(Message::OpenFolder(0)),
                        ]
                ],
                column![
                        text!("Save Folder: This is where Xaver will store backup saves. If the folder specified does not exist, Xaver will create it.")
                                .height(30),
                        row![
                                text_input(&self.save_folder, &self.save_folder_input)
                                        .size(15)
                                        .on_input(Message::SaveFolderChanged)
                                        .width(Length::FillPortion(15)),
                                button(
                                        create_img(&format!("folder.png"), image::FilterMethod::Linear).width(25)
                                )
                                .width(Length::FillPortion(1))
                                .style(button::text)
                                .height(30)
                                .on_press(Message::OpenFolder(0)),
                        ]
                ],
                button(text!("Create New Config").width(Length::Fill).center())
                    .on_press(Message::NewConfig)
                    .width(Length::Fill),
                button(text!("Refresh").width(Length::Fill).center())
                    .on_press(Message::Refresh)
                    .width(Length::Fill),
            )
            .padding(5)
            .spacing(5),
        );
        screening
    }
    // Sets up the folder variables to suggest common places for the files to be
    fn setup_folder_suggestions(&mut self) {
        self.game_folder = recommend_game_folder();
        self.save_folder = recommend_backup_folder();
    }

    /* ------------------ COMMON UTILITIES ------------------ */
    // Refreshes the page (reloads save info)
    fn refresh(&mut self) {
        // Gets the config and extracts data from it
        let data = get_config();
        match data {
            Ok(contents) => {
                if self.window_state == 2 {
                    self.window_state = 0;
                }
                self.game_folder = contents.gamedir;
                self.save_folder = contents.savedir;
                if fs::metadata(&self.game_folder).is_ok() {
                    self.game_saves = compile_info(&self.game_folder).unwrap_or(Vec::new());
                } else {
                    self.game_saves = Vec::new();
                }
                self.backup_saves.clear();
                for save in &self.game_saves {
                    if let Some((_, fname)) = save.path.rsplit_once('/') {
                        if fs::metadata(format!(
                            "{}/{}",
                            &self.save_folder,
                            fname.replace(".rsg", "")
                        ))
                        .is_ok()
                        {
                            self.backup_saves.append(
                                &mut compile_info(&format!(
                                    "{}/{}",
                                    &self.save_folder,
                                    fname.replace(".rsg", "")
                                ))
                                .unwrap_or(Vec::new()),
                            );
                        }
                    }
                }
                if let Some(selected) = self.selected_save.as_ref() {
                    if !self.game_saves.contains(selected) {
                        self.selected_save = None;
                        self.selected_save_path.clear();
                    } else {
                        self.find_unused_name(find_num_saves(
                            &self.backup_saves,
                            &self.selected_save_path,
                        ));
                    }
                }
            }
            Err(error) => {
                self.window_state = match error.kind() {
                    ErrorKind::NotFound => 2,
                    _ => self.window_state,
                };
                self.game_folder = String::new();
                self.save_folder = String::new();
                self.game_saves = Vec::new();
                self.backup_saves = Vec::new();
                self.selected_save = None;
                self.selected_save_path.clear();
            }
        }
    }
    // Finds a save name that hasn't been used yet
    fn find_unused_name(&mut self, num_saves: i64) {
        self.default_save_name = format!("Save {}", num_saves + 1);
        let x = fs::exists(format!(
            "{}/{}.rsg",
            &self.selected_save_path, &self.default_save_name
        ));
        if x.is_ok_and(|x| x == true) {
            self.find_unused_name(num_saves + 1);
        }
    }
    // Opens the file dialog for choosing new game/save folders
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
    // Generates an error message
    fn throw_error(&self, message: &String) -> Column<'_, Message> {
        column!(text!("{}", message).color(color!(255, 0, 0)))
            .width(Length::Fill)
            .padding(5)
    }
    // Makes an error message popup window
    fn error_popup(&self, message: String) -> Task<Message> {
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
}
