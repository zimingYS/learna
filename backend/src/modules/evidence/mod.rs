//! M05 学习证据：owner 级独立资源，v1 只开文本 / 链接三类（ADR-010 §4）。

pub mod dto;
pub mod handler;
pub mod rules;
pub mod service;

pub use handler::{create_evidence, delete_evidence, detail_evidence, list_evidence};
