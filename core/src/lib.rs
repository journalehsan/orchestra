/// Orchestra Core Library
///
/// Provides shared types, traits, and utilities for Orchestra applications.

pub mod error;
pub mod response;
pub mod pagination;

/// Re-export commonly used types
pub use error::{AppError, AppResult};
pub use response::ApiResponse;
