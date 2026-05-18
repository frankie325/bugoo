import { invoke } from "@tauri-apps/api/core";

export async function getSettings(): Promise<Record<string, string>> {
  return invoke("get_settings");
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export async function seedSettings(): Promise<void> {
  return invoke("seed_settings");
}