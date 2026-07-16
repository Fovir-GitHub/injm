use core::fmt;

use serde::Serialize;
use tabled::Tabled;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct MarkerBlock {
    pub begin_line: usize,
    pub end_line: usize,
    pub role: BlockRole,
}

pub enum BlockRole {
    Output { id: Option<String> },
    Input { ids: Vec<String>, content: String },
    Default,
}

#[derive(Serialize, Tabled)]
pub struct MarkerInfo {
    #[tabled(rename = "File")]
    pub file: String,

    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "Type")]
    pub marker_type: MarkerType,

    #[tabled(rename = "Lines")]
    pub lines: String,
}

pub struct ParsedFile {
    pub content: String,
    pub blocks: Vec<MarkerBlock>,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    Input,
    Output,
}

impl fmt::Display for MarkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkerType::Input => write!(f, "input"),
            MarkerType::Output => write!(f, "output"),
        }
    }
}
