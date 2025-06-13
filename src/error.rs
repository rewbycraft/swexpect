use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwitchExpectError {
    #[error("io error")]
    TokioIOError(#[from] tokio::io::Error),
    #[error("timeout while waiting for expect")]
    ExpectTimeout,
    #[error("unknown control code Ctrl+{0}")]
    UnknownControlCode(char),
}

pub type SwitchExpectResult<T> = Result<T, SwitchExpectError>;
