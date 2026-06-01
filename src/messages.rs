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
    OverwriteSave(String),
    RestoreSave(String),
    DeleteSave(String),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub savedir: String,
    pub backupdir: String,
}
