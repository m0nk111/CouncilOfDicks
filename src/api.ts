import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  ollama_url: string;
  ollama_model: string;
  debug_enabled: boolean;
}

export interface PerformanceMetrics {
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  average_response_time_ms: number;
}

export interface NetworkStatus {
  running: boolean;
  peer_id: string | null;
  connected_peers: number;
  port: number;
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

export async function getMetrics(): Promise<PerformanceMetrics> {
  return await invoke("get_metrics");
}

export async function p2pStart(): Promise<string> {
  return await invoke("p2p_start");
}

export async function p2pStop(): Promise<string> {
  return await invoke("p2p_stop");
}

export async function p2pStatus(): Promise<NetworkStatus> {
  return await invoke("p2p_status");
}
