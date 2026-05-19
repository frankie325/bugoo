import { invoke } from "@tauri-apps/api/core";

export async function speakText(
  text: string,
  lang?: string,
): Promise<void> {
  return invoke("speak_text", {
    text,
    lang: lang ?? null,
  });
}