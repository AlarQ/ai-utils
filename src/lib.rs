#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// Re-export commonly used types
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Module declarations
pub mod common;
pub mod error;
pub mod langfuse;
pub mod openai;
pub mod qdrant;
pub mod text_splitter;
