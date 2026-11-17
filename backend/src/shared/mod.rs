//! 共享层

pub mod error;
pub mod response;
pub mod types;

pub use error::{AppError, AppResult};
pub use response::{ApiResponse, CODE_OK};
pub use types::RequestId;
