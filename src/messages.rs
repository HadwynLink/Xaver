use crate::savemanager::SaveInfo;

#[derive(Debug, Clone)]
pub enum Message {
    SaveSelected(SaveInfo),
    Refresh,
    Launch,
    NewSave,
    SwitchDisplay(i32),
    OverwriteSave(String),
    RestoreSave(String),
    DeleteSave(String),
    SaveNameChanged(String),
    SaveFolderChanged(String),
    GameFolderChanged(String),
    OpenFolder(i32),
    FolderCanceled,
    FolderSelected(i32, String),
    ApplyFolder,
    MessageClosed,
    NewConfig,
}
