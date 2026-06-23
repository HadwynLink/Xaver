// Defines general utility functions for the upkeep of the tool
use crate::messages::*;
use iced::widget::{Container, center_x, image, text};
use iced::window::icon;
use serde::{Deserialize, Serialize};
use std::{env, fs, io::Error};

#[cfg(target_os = "linux")]
use users::{self, get_current_username};

#[cfg(target_os = "windows")]
use whoami;

// Data structure to store config information
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub gamedir: String,
    pub savedir: String,
}

// Returns the config file's information
pub fn get_config() -> Result<Config, Error> {
    let cfg_path: String = format!("{}/config/config.json", env::current_dir()?.display());
    let config = fs::read_to_string(&cfg_path)?;
    let data: Config = serde_json::from_str(&config)?;
    Ok(data)
}

// Saves new config info to the config file
pub fn save_config(info: Config) -> Result<(), Error> {
    let cfg_path: String = format!("{}/config/config.json", env::current_dir()?.display());
    let json_str = serde_json::to_string_pretty(&info)?;
    fs::write(&cfg_path, json_str)?;
    Ok(())
}

// Finds the most plausible location for the game folder on different platforms

pub fn recommend_game_folder() -> String {
    #[cfg(target_os = "linux")]
    {
        match get_current_username() {
            Some(uname) => {
                format!(
                    "/home/{}/.steam/steam/steamapps/compatdata/362490/pfx/drive_c/users/steamuser/Application Data/Exanima",
                    uname.to_string_lossy()
                )
            }
            None => "Username discovery error".to_string(),
        }
    }
    #[cfg(target_os = "windows")]
    {
        format!(
            "C:/Users/{}/AppData/Roaming/Exanima",
            whoami::username().unwrap_or_else(|_| "<unknown>".to_string()),
        )
    }
}

// Finds the most plausible location for the game folder on different platforms
pub fn recommend_backup_folder() -> String {
    match env::consts::OS {
        "linux" => match env::current_dir() {
            Ok(cur_dir) => {
                format!("{}/saves", cur_dir.to_string_lossy())
            }
            Err(_) => "Current Dir discovery error".to_string(),
        },
        "windows" => "Warning: Windows compatibility is completely untested!".to_string(),
        "macos" => "Warning: MacOS compatibility is completely untested!".to_string(),
        _ => "Warning: Possibly Unsupported Operating System!".to_string(),
    }
}

// Returns the path that should have the config in it
pub fn _get_proper_config_location() -> String {
    match env::current_dir() {
        Ok(cur_dir) => {
            format!("{}/config", cur_dir.display())
        }
        Err(_) => "Error: Could not find current directory!".to_string(),
    }
}

// Tries to make a new config file at the expected location
pub fn new_config(info: Config) -> Result<(), Error> {
    let cfg_path: String = format!("{}/config", env::current_dir()?.display());
    let json_str = serde_json::to_string_pretty(&info)?;
    fs::create_dir_all(&cfg_path)?;
    fs::write(&format!("{}/config.json", cfg_path), json_str)?;
    Ok(())
}

// Loads a window icon
pub fn load_icon() -> Result<Option<icon::Icon>, Error> {
    let cur_dir = env::current_dir()?;
    let bytes = std::fs::read(format!("{}/images/icon.png", cur_dir.to_string_lossy()))?;

    let icon =
        icon::from_file_data(&bytes, None).expect("Catastrophic error: Could not find icon!");
    Ok(Some(icon))
}

// Loads an image, or 404 if image is not found
pub fn create_img<'a>(path: &String, filter_method: image::FilterMethod) -> Container<'a, Message> {
    match env::current_dir() {
        Ok(dir) => {
            let banpath: String = format!("{}/images/{}", dir.display(), path);
            center_x(image(banpath).filter_method(filter_method))
        }
        Err(_) => center_x(text!("404")),
    }
}
