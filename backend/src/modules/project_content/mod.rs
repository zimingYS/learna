//! M03 项目内容上下文：内容加入项目、项目内状态与目标、跨项目复用查询。

pub mod dto;
pub mod handler;
pub mod rules;
pub mod service;

pub use dto::{ProjectContentItem, ProjectContentListData};
pub use handler::{
    create_project_content, delete_project_content, detail_project_content, list_project_contents,
    update_project_content,
};
