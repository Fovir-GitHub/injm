use std::path::PathBuf;

use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, CheckerError>;

#[derive(Debug, Error)]
pub(crate) enum CheckerError {
    #[error("file does not exist: {}", path.display())]
    FileNotExist { path: PathBuf },

    #[error("binary file detected: {}",path.display())]
    BinaryFile { path: PathBuf },

    #[error("missing input id `{id}`")]
    MissingInputID { id: String },

    #[error("duplicated input id `{id}`")]
    DuplicatedInputID { id: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
