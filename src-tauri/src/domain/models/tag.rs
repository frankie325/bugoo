use serde::{Deserialize, Serialize};

/// 标签实体 - 与前端 TagItem 类型对齐
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String, // HEX format: "#RRGGBB"
    #[serde(rename = "sort_order")]
    pub sort_order: i64, // 排序权重，越小越靠前
    pub created_at: i64, // Unix timestamp (ms)
    pub updated_at: i64,
}

/// 创建标签输入 - 与前端 TagCreateInput 对齐
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TagCreateInput {
    pub name: String,
    pub color: String,
    pub sort_order: Option<i64>,
}

/// 更新标签输入 - 与前端 TagUpdateInput 对齐（全可选）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct TagUpdateInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i64>,
}

/// 重排标签输入 - 与前端 TagReorderInput 对齐
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TagReorderInput {
    pub tag_ids: Vec<String>,
}

impl Tag {
    /// 创建新标签（生成默认值）
    pub fn new(input: TagCreateInput) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id: id.to_string(),
            name: input.name,
            color: input.color,
            sort_order: input.sort_order.unwrap_or(now),
            created_at: now,
            updated_at: now,
        }
    }

    /// 应用更新（不可变模式，返回新对象）
    pub fn apply_update(self, input: TagUpdateInput) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            name: input.name.unwrap_or(self.name),
            color: input.color.unwrap_or(self.color),
            sort_order: input.sort_order.unwrap_or(self.sort_order),
            id: self.id,
            created_at: self.created_at,
            updated_at: now,
        }
    }
}
