//! M05 学习活动：正式 LearningSession + 三类关联整体替换。
//!
//! 无状态机、无服务端计时（ADR-010）：写入即事实，计时状态完全在客户端。

pub mod associations;
pub mod dto;
pub mod handler;
pub mod rules;
pub mod service;

pub use handler::{
    create_session, delete_session, detail_session, list_sessions, set_session_contents,
    set_session_projects, set_session_tasks, update_session,
};
