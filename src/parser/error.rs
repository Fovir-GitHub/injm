use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug, Error)]
pub(crate) enum ParserError {
    #[error(transparent)]
    Process(#[from] tree_sitter_language_pack::Error),

    #[error(transparent)]
    Checker(#[from] crate::checker::CheckerError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Glob(#[from] glob::GlobError),

    #[error(transparent)]
    Pattern(#[from] glob::PatternError),

    #[error("unsupported file type: {path}")]
    UnsupportedFileType { path: String },

    #[error("found nested `injm begin` without `injm end` at line {line} of {path}")]
    NestedMarker { line: usize, path: String },

    #[error("found `injm end` without `injm begin` at line {line} of {path}")]
    UnclosedMarker { line: usize, path: String },

    #[error("found both input and output ID: {comment}")]
    BothInputOutputMarker { comment: String },

    #[error("multiple output IDs detected: {comment}")]
    MultipleOutputMarker { comment: String },

    #[error("no files matched pattern `{pattern}`")]
    NoPatternMatch { pattern: String },
}
