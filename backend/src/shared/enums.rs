//! 领域状态枚举集中定义。
//!
//! 三个理由让它们必须集中在一处：DB `CHECK` 约束、请求反序列化校验、响应序列化字面值
//! 必须是同一份真理；分散到各模块的 dto 里，三者迟早漂移成三套拼写。

use serde::{Deserialize, Serialize};

use crate::shared::{AppError, AppResult};

/// 生成「字面值 + serde + DB 双向转换」三件套的状态枚举。
///
/// DB 列是 `VARCHAR`，读取时得到 `String`；`parse` 的失败分支返回 `Internal` 而不是
/// `Validation`——走到那里说明 `CHECK` 约束被绕过或迁移与代码不一致，是服务端缺陷。
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $value:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $value)] $variant),+
        }

        impl $name {
            /// 数据库与线上契约使用的字面值
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }

            /// 从数据库字面值还原；未知值视为服务端缺陷
            pub fn parse(value: &str) -> AppResult<Self> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    other => Err(AppError::Internal(format!(
                        concat!("unknown {} value in database: {}"),
                        stringify!($name),
                        other
                    ))),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_enum! {
    /// 项目状态（六值合并，D-16）。`Deleting` 只由 purge 事务内部写入。
    ProjectStatus {
        Draft => "draft",
        Active => "active",
        Paused => "paused",
        Completed => "completed",
        Archived => "archived",
        Deleting => "deleting",
    }
}

string_enum! {
    /// 项目类型（ERD §5.1）
    ProjectType {
        Study => "study",
        SkillDevelopment => "skill_development",
        Challenge => "challenge",
        Deliverable => "deliverable",
    }
}

string_enum! {
    /// 截止日期类型；`none` 时 `deadline_at` 必须为空
    DeadlineType {
        None => "none",
        Soft => "soft",
        Fixed => "fixed",
    }
}

string_enum! {
    /// 目标类型
    GoalType {
        Outcome => "outcome",
        Performance => "performance",
        Completion => "completion",
    }
}

string_enum! {
    /// 目标状态；`achieved` 是成果，禁止删除也禁止直转 `cancelled`
    GoalStatus {
        Active => "active",
        Achieved => "achieved",
        Cancelled => "cancelled",
    }
}

string_enum! {
    /// 目标-阶段关联的贡献程度（ERD GoalStage）
    Contribution {
        Primary => "primary",
        Supporting => "supporting",
    }
}

string_enum! {
    /// 路线状态；同一项目最多一个 `active`
    RoadmapStatus {
        Draft => "draft",
        Active => "active",
        Superseded => "superseded",
    }
}

string_enum! {
    /// 路线来源
    RoadmapSource {
        User => "user",
        Ai => "ai",
        Template => "template",
    }
}

string_enum! {
    /// 阶段状态。走动作端点而非 PATCH，`pending` 不可直达 `completed`。
    StageStatus {
        Pending => "pending",
        Active => "active",
        Completed => "completed",
        Skipped => "skipped",
    }
}

string_enum! {
    /// 完成确认方式；v1 只产生 `user`
    CompletionMethod {
        User => "user",
        SystemConfirmed => "system_confirmed",
        AiSuggested => "ai_suggested",
    }
}

string_enum! {
    /// 里程碑状态
    MilestoneStatus {
        Pending => "pending",
        InProgress => "in_progress",
        Completed => "completed",
        Skipped => "skipped",
    }
}

string_enum! {
    /// 任务状态（五值）
    TaskStatus {
        Todo => "todo",
        InProgress => "in_progress",
        Blocked => "blocked",
        Completed => "completed",
        Cancelled => "cancelled",
    }
}

string_enum! {
    /// 创建来源
    CreatedSource {
        User => "user",
        Ai => "ai",
        Template => "template",
    }
}

string_enum! {
    /// 任务依赖类型；ERD §5.1 仅定义 finish_to_start
    DependencyType {
        FinishToStart => "finish_to_start",
    }
}
