use core::fmt;

use serde::Serialize;
use tabled::Tabled;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Comment {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct MarkerBlock {
    pub begin_line: usize,
    pub end_line: usize,

    // Allow multiple input markers,
    // while a block can have at most one output marker.
    pub input_ids: Vec<String>,
    pub input_content: Option<String>,
    pub output_id: Option<String>,
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
