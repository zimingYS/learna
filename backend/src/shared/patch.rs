//! PATCH 三态语义（ADR-002）的序列化支撑。
//!
//! JSON 里「字段缺失」与「字段显式为 null」在 `Option<T>` 上是同一个 `None`，
//! 而 ADR-002 要求它们是两种不同意图：缺失＝不变、null＝置空、值＝更新。
//! 用 `Option<Option<T>>` 表达三态，缺的那一半靠这里的 `deserialize_with` 补上。

use serde::{Deserialize, Deserializer};

/// 反序列化为 `Some(...)`，从而让「字段缺失」落在 `#[serde(default)]` 的 `None` 上。
///
/// 用法：
/// ```ignore
/// #[serde(default, deserialize_with = "double_option")]
/// pub description: Option<Option<String>>,
/// ```
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
