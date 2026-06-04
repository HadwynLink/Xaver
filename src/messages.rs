use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    Load,
    Checkpoint,
    SaveSelected(String),
    Refresh,
    Launch,
    Settings,
    NewSave,
    OverwriteSave(usize),
    RestoreSave(usize),
    DeleteSave(usize),
    ContentChanged(String),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub savedir: String,
    pub backupdir: String,
}

pub struct SaveInfo {
    pub path: String,
    pub name: String,
    pub location: String,
}
