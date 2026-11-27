//! M02 学习项目管理：项目本体、生命周期、归档与受控删除。

pub mod dto;
pub mod handler;
pub mod rules;
pub mod service;

pub use dto::{ProjectItem, ProjectListData};
pub use handler::{
    activate, archive, complete, create, delete, detail, extend_deadline, list, pause, purge,
    reopen, resume, update,
};
pub use rules::ProjectAction;
