import { invoke } from "@tauri-apps/api/core";
import type {
  TagItem,
  TagCreateInput,
  TagUpdateInput,
  TagReorderInput,
} from "../../types/tag";

const STORAGE_KEY = "bugoo:tags";

function getLocalTags(): TagItem[] {
  const saved = localStorage.getItem(STORAGE_KEY);
  return saved ? JSON.parse(saved) : [];
}

function setLocalTags(tags: TagItem[]): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags));
}

function generateId(): string {
  return `tag_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

/**
 * 获取所有标签（按 sort_order 升序）
 * 后端未就绪时自动降级到 localStorage
 */
export async function getTags(): Promise<TagItem[]> {
  try {
    return await invoke<TagItem[]>("get_tags");
  } catch (e) {
    console.error("[tags] invoke('get_tags') failed, falling back to localStorage:", e);
    return getLocalTags().sort((a, b) => a.sort_order - b.sort_order);
  }
}

export async function createTag(input: TagCreateInput): Promise<TagItem> {
  try {
    return await invoke<TagItem>("create_tag", { input });
  } catch {
    const newTag: TagItem = {
      id: generateId(),
      name: input.name,
      color: input.color,
      sort_order: input.sort_order ?? Date.now(),
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    const tags = getLocalTags();
    tags.push(newTag);
    setLocalTags(tags);
    return newTag;
  }
}

export async function updateTag(
  id: string,
  input: TagUpdateInput,
): Promise<TagItem> {
  try {
    return await invoke<TagItem>("update_tag", { id, input });
  } catch {
    const tags = getLocalTags();
    const index = tags.findIndex((t) => t.id === id);
    if (index === -1) throw new Error(`Tag ${id} not found`);
    tags[index] = { ...tags[index], ...input, updated_at: Date.now() };
    setLocalTags(tags);
    return tags[index];
  }
}

export async function deleteTag(id: string): Promise<void> {
  try {
    await invoke<void>("delete_tag", { id });
  } catch {
    const tags = getLocalTags();
    const filtered = tags.filter((t) => t.id !== id);
    setLocalTags(filtered);
  }
}

export async function reorderTags(input: TagReorderInput): Promise<TagItem[]> {
  try {
    return await invoke<TagItem[]>("reorder_tags", { input });
  } catch {
    const tags = getLocalTags();
    const tagMap = new Map(tags.map((t) => [t.id, t]));
    const reordered = input.tag_ids.map((id, index) => {
      const tag = tagMap.get(id);
      if (!tag) throw new Error(`Tag ${id} not found`);
      return { ...tag, sort_order: index, updated_at: Date.now() };
    });
    setLocalTags(reordered);
    return reordered;
  }
}