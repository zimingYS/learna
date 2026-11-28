//! M03 学习内容域：内容本体 / 内容关系 / 标签词表。
//!
//! 项目内的内容上下文在 `modules::project_content`：内容归属是用户级（`owner_id`），
//! 「学到哪一步」是项目级，两者合并会让项目状态字段被多个项目共享（ERD §3.2）。

pub mod dto;
pub mod handler;
pub mod relation;
pub mod rules;
pub mod service;
pub mod tag;

pub use dto::{ContentCreateData, ContentDetailItem, ContentItem, ContentListData};
pub use handler::{
    create_content, create_relation, create_tag, delete_content, delete_relation, delete_tag,
    detail_content, list_content_tags, list_contents, list_relations, list_tags, set_content_tags,
    update_content, update_tag,
};
