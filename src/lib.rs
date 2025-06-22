#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// Re-export commonly used types
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Module declarations
pub mod common;
pub mod error;

#[cfg(feature = "langfuse")]
pub mod langfuse;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "qdrant")]
pub mod qdrant;

#[cfg(feature = "text-splitter")]
pub mod text_splitter;
