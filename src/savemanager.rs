use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::Command;
use std::str;

// load saves into data structure to call on later
pub fn compile_saves(from: &String) -> Vec<String> {
    let mut saves: Vec<String> = Vec::new();
    let paths = fs::read_dir(from).unwrap();
    for path in paths {
        let fname: String = format!("{}", path.unwrap().path().display());
        if fname.contains(".rsg") {
            let fpruned: String = fname.replace(from.as_str(), "");
            saves.push(fpruned);
        }
    }
    return saves;
}

// Lists the .rsg files in a directory
pub fn list_files(list: &Vec<String>) {
    let mut i = 1;
    for save in list {
        println!("{}: {}", i, save.replace(".rsg", ""));
        i += 1;
    }
}

// Creates a new save, making a new directory for it if it doesn't exist
pub fn new_save(from: &String, savedir: &String, savename: &String) {
    fs::create_dir_all(savedir).expect("Could not create directory");
    fs::copy(from, format!("{}/{}.rsg", savedir, savename)).expect("File could not copy!");
}

// Copies a given save file into the backups folder
pub fn copy_save(from: &String, to: &String) {
    fs::copy(from, to).expect("File could not copy!");
}

// Outputs the name of the character in the file
pub fn read_name(tar: &String) -> String {
    let output = Command::new("strings") // This can chew up a lot of resources. Need to find a faster way.
        .arg(tar)
        .output()
        .expect("failed to execute process")
        .stdout;

    let stringout = str::from_utf8(&output).expect("Could not extract string");

    let mut foundname = false;
    let mut name = String::new();

    let name_marker = if tar.contains("Arena") {
        "arenahub"
    } else {
        "0a\"A"
    };

    for line in stringout.lines() {
        if foundname == true {
            name = format!("{}", line);
            break;
        } else {
            if line.contains(name_marker) {
                foundname = true;
            }
        }
    }
    name
}

// Grabs both name and location in one command.
pub fn read_info(tar: &String) -> Vec<String> {
    let output = Command::new("strings") // This can chew up a lot of resources. Need to find a faster way.
        .arg(tar)
        .output()
        .expect("failed to execute process")
        .stdout;

    let stringout = str::from_utf8(&output).expect("Could not extract string");

    let mut levelraw = String::new();
    if tar.contains("Arena") {
        levelraw = format!("arenahub") // This will always be the case for arena saves
    }
    let mut name = String::new();
    let name_marker = if tar.contains("Arena") {
        "arenahub"
    } else {
        "0a\"A"
    };

    let mut foundname = false;
    for line in stringout.lines() {
        if levelraw.is_empty() || name.is_empty() {
            if foundname == true && name.is_empty() {
                name = format!("{}", line);
            } else if line.contains(name_marker) {
                foundname = true;
            } else if line.contains("Zoe") && levelraw.is_empty() {
                levelraw = format!("{}", line);
            }
        } else {
            break;
        }
    }

    let level;
    if levelraw.contains("arenahub") {
        level = format!("Arena Hub");
    } else if levelraw.contains("01") {
        level = format!("Level 1");
    } else if levelraw.contains("02") {
        level = format!("Level 2");
    } else if levelraw.contains("c1") {
        level = format!("Level 2.5\n(The Catacombs)");
    } else if levelraw.contains("03") {
        level = format!("Level 3");
    } else if levelraw.contains("04") {
        level = format!("Level 4\n(The Archives)");
    } else if levelraw.contains("05_sw") {
        level = format!("Level 5.5\n(The Crossroads Sewers)");
    } else if levelraw.contains("05") {
        level = format!("Level 5\n(The Crossroads)");
    } else if levelraw.contains("06") {
        level = format!("Level 6\n(The Forge)");
    } else if levelraw.contains("07_sw") {
        level = format!("Level 7.5\n(The Market Sewers)");
    } else if levelraw.contains("07") {
        level = format!("Level 7\n(The Market)");
    } else if levelraw.contains("08") {
        level = format!("Level 8\n(The Gentry)");
    } else if levelraw.contains("67") {
        level = format!("Level 9\n(The Gardens)");
    } else {
        level = format!("Unknown Area! Level ID: {}", levelraw);
    }

    vec![name, level]
}

// Returns the current level the character is on
pub fn read_level(tar: &String) -> String {
    let mut text = String::new();
    if tar.contains("Arena") {
        text = format!("arenahub"); // This will always be the case for the arena
    } else {
        let output = Command::new("strings")
            .arg(tar)
            .output()
            .expect("failed to execute process")
            .stdout;

        let stringout = str::from_utf8(&output).expect("Could not extract string");

        for line in stringout.lines() {
            // The first instance of this key seems to be the right one
            if line.contains("Zoe") {
                text = format!("{}", line);
                break;
            }
        }
    }

    let mut answer = String::new();
    // This is a *nasty* if-else loop. Very icky. No good.
    // I could probably do this much better with a dictionary but I would need to clean the text better first...
    if text.contains("arenahub") {
        answer = format!("Arena Hub");
    } else if text.contains("01") {
        answer = format!("Level 1");
    } else if text.contains("02") {
        answer = format!("Level 2");
    } else if text.contains("c1") {
        answer = format!("Level 2.5\n(The Catacombs)");
    } else if text.contains("03") {
        answer = format!("Level 3");
    } else if text.contains("04") {
        answer = format!("Level 4\n(The Archives)");
    } else if text.contains("05_sw") {
        answer = format!("Level 5.5\n(The Crossroads Sewers)");
    } else if text.contains("05") {
        answer = format!("Level 5\n(The Crossroads)");
    } else if text.contains("06") {
        answer = format!("Level 6\n(The Forge)");
    } else if text.contains("07_sw") {
        answer = format!("Level 7.5\n(The Market Sewers)");
    } else if text.contains("07") {
        answer = format!("Level 7\n(The Market)");
    } else if text.contains("08") {
        answer = format!("Level 8\n(The Gentry)");
    } else if text.contains("67") {
        answer = format!("Level 9\n(The Gardens)");
    } else {
        answer = format!("Unknown Area! Level ID: {}", text);
    }

    answer
}

// Generates fancy labels for saves instead of raw file names
pub fn generate_save_display(at: &String, saves: &Vec<String>) -> Vec<String> {
    let mut savedisps = Vec::new();

    for save in saves {
        let mut save_display = String::new();
        if save.contains("Arena") {
            save_display.push_str(&format!(
                "Arena Save {} ",
                save.replace("Arena", "").replace(".rsg", "")
            ));
        } else {
            save_display.push_str(&format!(
                "Campaign Save {} ",
                save.replace("Exanima", "").replace(".rsg", "")
            ));
        }
        save_display.push_str(&format!("({})", read_name(&format!("{}{}", at, save))));
        savedisps.push(save_display);
    }

    savedisps
}
