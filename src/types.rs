pub struct MarkerBlock {
    pub span: SourceSpan,
    pub role: BlockRole,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub begin_marker: usize,
    pub end_marker: usize,
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

impl SourceSpan {
    pub fn new(begin_marker: usize, end_marker: usize) -> Self {
        Self {
            begin_marker,
            end_marker,
        }
    }

    pub fn content_lines(&self) -> std::ops::Range<usize> {
        self.begin_marker + 1..self.end_marker
    }

    pub fn display_lines(&self) -> String {
        format!("{}-{}", self.begin_marker + 1, self.end_marker + 1)
    }

    pub fn before_lines(&self) -> std::ops::RangeToInclusive<usize> {
        ..=self.begin_marker
    }

    pub fn after_lines(&self) -> std::ops::RangeFrom<usize> {
        self.end_marker..
    }
}
