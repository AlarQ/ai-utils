use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Directory read failed: {0}")]
    DirectoryRead(String),

    #[error("File read failed: {0}")]
    FileRead(String),

    #[error("No valid image files found in directory: {0}")]
    NoValidFiles(String),
}
