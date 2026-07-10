pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub type OutputID = Option<String>;

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
    pub output_id: OutputID,
}

pub struct ParsedFile {
    pub content: String,
    pub blocks: Vec<MarkerBlock>,
}
