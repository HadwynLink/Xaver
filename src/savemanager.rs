use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

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

// Copies a given save file into the backups folder
pub fn copy_save(from: &String, to: &String, tar: &String) {
    let mut fullpath: String = format!("{}", from);
    fullpath.push_str(tar);
    let mut topath: String = format!("{}", to);
    topath.push_str(tar);
    println!("{}", fullpath);
    fs::copy(fullpath, topath).expect("File could not copy!");
    println!("Successfully copied {tar} to {to}");
}

// Copies the given save file, modifies the extension, and overwrites the checkpoint of that name
pub fn checkpointify(at: &String, tar: &String) {
    let mut fullpath: String = format!("{}", at);
    fullpath.push_str(tar);
    let mut topath: String = format!("{}", at);
    topath.push_str(&tar.replace(".rsg", ".rcp"));
    println!("{}", fullpath);
    fs::copy(fullpath, topath).expect("File could not copy!");
    println!("Successfully checkpointed {tar}");
}

// Outputs the name of the character in the file
pub fn read_name(at: &String, tar: &String) -> String {
    let mut startchar = 8262; // Position of the name in memory
    let mut endchar = 8280;
    if tar.contains("Arena") {
        // Arena saves store names differently
        startchar = 8390;
        endchar = 8408;
    }
    let mut fullpath: String = format!("{}", at);
    fullpath.push_str(tar);

    let file = File::open(fullpath).expect("Couldn't find file");
    let mut reader = BufReader::new(file);

    let mut line = vec![0u8; 9000];
    reader.read(&mut line).expect("Couldn't read file");

    let slice = &line[startchar..endchar];

    let text = String::from_utf8_lossy(slice)
        .chars()
        .filter(|&c| c != '\u{FFFD}')
        .collect();

    text
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
        save_display.push_str(&format!("({})", read_name(at, save)));
        savedisps.push(save_display);
    }

    savedisps
}
