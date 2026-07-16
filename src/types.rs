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

pub struct ParsedFile {
    pub content: String,
    pub blocks: Vec<MarkerBlock>,
    pub path: String,
}
