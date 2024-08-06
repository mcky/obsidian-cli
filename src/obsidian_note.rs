use std::path::PathBuf;

pub type Properties = serde_yaml::Value;

#[derive(Debug, PartialEq, Eq)]
pub struct ObsidianNote {
    pub file_path: PathBuf,
    pub file_contents: String,
    pub file_body: String,
    pub properties: Option<Properties>,
}
