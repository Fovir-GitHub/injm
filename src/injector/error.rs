use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, InjectorError>;

#[derive(Debug, Error)]
pub(crate) enum InjectorError {
    #[error("empty input content")]
    EmptyInputContent,
}
