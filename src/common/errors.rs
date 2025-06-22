use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Invalid file extension: {0}")]
    InvalidExtension(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Directory read failed: {0}")]
    DirectoryRead(String),

    #[error("File read failed: {0}")]
    FileRead(String),

    #[error("Base64 encoding failed: {0}")]
    Base64Encoding(String),

    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    #[error("File size exceeds limit: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },

    #[error("No valid image files found in directory: {0}")]
    NoValidFiles(String),
}
