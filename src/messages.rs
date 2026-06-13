use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Message {
    SaveSelected(String),
    Refresh,
    Launch,
    Settings,
    NewSave,
    OverwriteSave(usize),
    RestoreSave(usize),
    DeleteSave(usize),
    ContentChanged(String),
    MessageClosed,
    OpenFolder(i32),
    FolderCanceled,
    FolderSelected(i32, String),
    ApplyFolder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub savedir: String,
    pub backupdir: String,
}

pub struct SaveInfo {
    pub path: String,
    pub name: String,
    pub location: String,
}
