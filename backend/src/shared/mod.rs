//! 共享层

pub mod error;
pub mod middleware;
pub mod response;
pub mod types;

pub use error::{AppError, AppResult};
pub use response::{ApiReply, ApiResponse, CODE_OK};
pub use types::RequestId;
