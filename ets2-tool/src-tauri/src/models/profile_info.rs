use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProfileInfo {
    pub path: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum SaveKind {
    Manual,
    Autosave,
    Invalid,
}
