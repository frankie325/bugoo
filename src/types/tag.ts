export interface TagItem {
  id: string;
  name: string;
  color: string; // HEX format: "#RRGGBB"
  sort_order: number; // 排序权重，越小越靠前
  created_at: number; // Unix timestamp (ms)
  updated_at: number;
}

export interface TagCreateInput {
  name: string;
  color: string;
  sort_order?: number;
}

export interface TagUpdateInput {
  name?: string;
  color?: string;
  sort_order?: number;
}

export interface TagReorderInput {
  tag_ids: string[]; // snake_case 与 Rust 字段对齐
}