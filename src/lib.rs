#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::unused_async)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

// Re-export commonly used types
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Module declarations
pub mod common;
pub mod error;
pub mod langfuse;
pub mod openai;
pub mod text_splitter;
pub mod qdrant;
