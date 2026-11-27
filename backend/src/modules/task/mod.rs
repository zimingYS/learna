//! M04 学习任务与依赖：软删除、状态迁移、前置依赖图。

pub mod dto;
pub mod graph;
pub mod handler;
pub mod rules;
pub mod service;

pub use dto::{TaskItem, TaskListData};
pub use handler::{
    create, create_dependency, delete, delete_dependency, detail, list, list_dependencies, update,
};
