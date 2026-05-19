import { invoke } from "@tauri-apps/api/core";

export interface VoiceInfo {
  id: string;
  name: string;
  language: string;
}

export async function speakText(text: string, lang?: string): Promise<void> {
  return invoke("speak_text", {
    text,
    lang: lang ?? null,
  });
}

export async function stopSpeech(): Promise<void> {
  return invoke("stop_speech");
}

export async function listVoices(): Promise<VoiceInfo[]> {
  return invoke("list_voices");
}

export async function setVoice(voiceId: string): Promise<void> {
  return invoke("set_voice", { voiceId });
}