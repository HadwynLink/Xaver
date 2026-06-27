// Defines functions for working with saves
use chrono::{DateTime, Local};
use memchr::memmem;
use std::{
    collections::HashMap,
    fmt,
    fs::{self, metadata},
    io::Error,
    sync::LazyLock,
};

// Dictionary to store names of levels
static LEVEL_DICT: LazyLock<HashMap<&'static str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("arenahub", "Arena Hub"),
        ("Zoexanima01", "Level 1"),
        ("Zoexanima02", "Level 2"),
        ("Zoexanimac1", "Level 2.5\n(The Catacombs)"),
        ("Zoexanima03", "Level 3"),
        ("Zoexanima04", "Level 4\n(The Archives)"),
        ("Zoexanima05", "Level 5\n(The Crossroads)"),
        ("Zoexanima05_sw", "Level 5.5\n(The Crossroads Sewers)"),
        ("Zoexanima06", "Level 6\n(The Golem Forge)"),
        ("Zoexanima07", "Level 7\n(The Market)"),
        ("Zoexanima07_sw", "Level 7.5\n(The Market Sewers)"),
        ("Zoexanima08", "Level 8\n(The Gentry)"),
        ("Zoexanima67", "Level 9\n(The Gardens)"),
        ("Zoexanima67_c1", "Level 9.5\n(The Garden Crypts)"),
    ])
});

// Data structure for save information
#[derive(Debug, Default, Clone)]
pub struct SaveInfo {
    pub path: String,                   // Save file location
    pub save_type: i32,                 // Save file type: Campaign(0) vs Arena(1)
    pub created_date: DateTime<Local>,  // Date the save was created
    pub modified_date: DateTime<Local>, // Date the save was last modified
    pub file_size: f64,                 // Size of the file (MB)
    pub name: String,                   // Name of the character
    pub level: String,                  // Location of the character
}

impl PartialEq for SaveInfo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl fmt::Display for SaveInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display_name = String::new();
        match self.save_type {
            0 => {
                if let Some((_, fname)) = self.path.replace("\\", "/").rsplit_once('/') {
                    display_name.push_str(&format!(
                        "Campaign Save {}: {}",
                        fname.replace(".rsg", "").replace("Exanima", ""),
                        self.name
                    ));
                    if self.created_date != DateTime::<Local>::default() {
                        display_name.push_str(&format!(
                            " (Created {})",
                            self.created_date.format("%m/%d/%Y")
                        ))
                    } else {
                        display_name.push_str(&format!(" (Unknown Creation Date)"))
                    }
                } else {
                    display_name.push_str("Unreadable Campaign Save");
                }
            }
            1 => {
                if let Some((_, fname)) = self.path.replace("\\", "/").rsplit_once('/') {
                    display_name.push_str(&format!(
                        "Arena Save {}: {}",
                        fname.replace(".rsg", "").replace("Arena", ""),
                        self.name
                    ));
                    if self.created_date != DateTime::<Local>::default() {
                        display_name.push_str(&format!(
                            " (Created {})",
                            self.created_date.format("%m/%d/%Y")
                        ))
                    } else {
                        display_name.push_str(&format!(" (Unknown Creation Date)"))
                    }
                } else {
                    display_name.push_str("Unreadable Arena Save");
                }
            }
            _ => {
                display_name.push_str("Unknown Save Type");
            }
        }
        write!(f, "{}", display_name)
    }
}

// Compiles a list of save information
pub fn compile_info(from: &String) -> Result<Vec<SaveInfo>, Error> {
    let mut saves: Vec<SaveInfo> = Vec::new();
    let paths = fs::read_dir(from)?;
    for path in paths {
        let fname: String = format!("{}", path?.path().display());
        if fname.contains(".rsg") {
            let fpruned: SaveInfo = read_info(&fname)?;
            saves.push(fpruned);
        }
    }
    Ok(saves)
}

// Creates the path to the save folder if it doesn't exist already
pub fn create_save_folder(path: &String) -> Result<(), Error> {
    if !metadata(path).is_ok() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

// Creates a new save, making a new directory for it if it doesn't exist
pub fn new_save(from: &String, savedir: &String, savename: &String) -> Result<(), Error> {
    fs::create_dir_all(savedir)?;
    fs::copy(from, format!("{}/{}.rsg", savedir, savename))?;
    Ok(())
}

// Copies a given save file into the backups folder
pub fn copy_save(from: &String, to: &String) -> Result<(), Error> {
    fs::copy(from, to)?;
    Ok(())
}

// Returns relevant information of a target save.
pub fn read_info(tar: &String) -> Result<SaveInfo, Error> {
    let mut info = SaveInfo::default();
    info.path = tar.to_string();

    match fs::metadata(tar) {
        Ok(metadata) => {
            match metadata.created() {
                Ok(date) => {
                    info.created_date = date.into();
                }
                Err(_) => {}
            };
            match metadata.modified() {
                Ok(date) => {
                    info.modified_date = date.into();
                }
                Err(_) => {}
            }
            info.file_size = (metadata.len() as f64) / 1000000.0;
        }
        Err(_) => {}
    };

    let mut levelraw = String::new();
    let data = std::fs::read(tar)?;

    if memmem::find(&data[0..500], b"arenahub") != None {
        info.save_type = 1;
    }

    if info.save_type == 1 {
        levelraw = "arenahub".to_string(); // This will always be the case for arena saves
    } else if let Some(pos) = memmem::find(&data, b"Zoexanima") {
        let start = pos;

        let mut end = pos;
        while end < data.len() && data[end].is_ascii_graphic() {
            end += 1;
        }
        levelraw = format!("{}", String::from_utf8_lossy(&data[start..end]));
    }

    if info.save_type == 0 {
        let player_pattern = [0xCD, 0xAC, 0xDB]; // First check pattern for finding the player data
        let name_pattern = [0xAC, 0xCD, 0x00]; // For finding the name specifically
        let mut found_spot = false;
        let mut first_layer_pos = 0;
        while found_spot == false {
            if let Some(pos) = memmem::find(&data[first_layer_pos..], &player_pattern)
                && data.len() > first_layer_pos
            {
                if (data[first_layer_pos + pos + 3] == 0x20
                    && data[first_layer_pos + pos + 4] == 0x00)
                    || (data[first_layer_pos + pos + 3] == 0x30
                        && data[first_layer_pos + pos + 4] == 0x00)
                    || (data[first_layer_pos + pos + 3] == 0x40
                        && data[first_layer_pos + pos + 4] == 0x00)
                {
                    found_spot = true;
                    if let Some(pos_2) = memmem::find(&data[first_layer_pos + pos..], &name_pattern)
                    {
                        let mut start = first_layer_pos + pos + pos_2 + 7;
                        while start < data.len() && !(0x20..=0x7E).contains(&data[start]) {
                            start += 1;
                        }
                        let mut end = start;
                        while end < data.len() && (0x20..=0x7E).contains(&data[end]) {
                            end += 1;
                        }

                        info.name = format!("{}", String::from_utf8_lossy(&data[start..end]));
                    }
                } else {
                    first_layer_pos = first_layer_pos + pos + 11;
                }
            } else {
                found_spot = true;
            }
        }
    } else {
        let start = 8392;

        let mut end = 8392;
        while end < data.len() && (0x20..=0x7E).contains(&data[end]) {
            end += 1;
        }

        info.name = format!("{}", String::from_utf8_lossy(&data[start..end]));
    }
    if let Some(lvl) = LEVEL_DICT.get(levelraw.as_str()) {
        info.level = format!("{}", lvl);
    } else {
        info.level = format!("Unknown Level!\nId = {}", levelraw);
    }
    Ok(info)
}

// Finds how many saves in a vector are related to a selected save
pub fn find_num_saves(saves: &Vec<SaveInfo>, path: &String) -> i64 {
    let mut num_saves = 0;
    for save in saves {
        if save.path.contains(path) {
            num_saves += 1;
        }
    }
    num_saves
}
