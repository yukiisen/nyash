#![allow(unused)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadDirError {
    #[error("Failed to open directory")]
    OpenDirError(String),

    #[error("Failed to open directory: no such path")]
    DirectoryNotFound(String),

    #[error("Failed to read directory")]
    ReadDirectoryError(String),

    #[error("Unknown Error")]
    Unknown(String),
}
