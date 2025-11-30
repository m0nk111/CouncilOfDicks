import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  ollama_url: string;
  ollama_model: string;
  debug_enabled: boolean;
}

export async function askCouncil(question: string): Promise<string> {
  return await invoke("ask_ollama", { question });
}

export async function getConfig(): Promise<AppConfig> {
  return await invoke("get_config");
}

export async function setDebug(enabled: boolean): Promise<void> {
  return await invoke("set_debug", { enabled });
}
