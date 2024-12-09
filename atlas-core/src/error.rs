use serde::de::Error as SerdeError;
use serde::ser::Error as SerdeSerError;
use std::{fs, io};
use thiserror::Error;
use tokio::task::JoinError;
use toml;

//==========================================================================
#[derive(Error, Debug)]
pub enum AtlasError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML deserialization error: {0}")]
    Log(#[from] log::SetLoggerError),

    #[error("Tokio task join error: {0}")]
    JoinError(JoinError),
}

//==========================================================================
impl AtlasError {}

//==========================================================================
impl From<JoinError> for AtlasError {
    fn from(err: JoinError) -> Self {
        AtlasError::JoinError(err)
    }
}

//==========================================================================
pub type AtlasResult<T> = Result<T, AtlasError>;
