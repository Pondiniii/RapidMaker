use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("not implemented: {capability}")]
    NotImplemented { capability: &'static str },

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),

    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),

    #[error("parse error: {0}")]
    Parse(String),
}
