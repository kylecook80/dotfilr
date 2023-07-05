use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not find config file {0}")]
    ConfigOpen(PathBuf),

    #[error("could not parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),
}
