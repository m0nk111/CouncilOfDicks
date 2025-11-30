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

// Council session types
export interface CouncilResponse {
  model_name: string;
  response: string;
  peer_id: string;
  timestamp: number;
}

export interface VoteCommitment {
  commitment_hash: string;
  voter_peer_id: string;
}

export interface VoteReveal {
  vote: string;
  salt: string;
  voter_peer_id: string;
}

export type SessionStatus = 
  | "GatheringResponses"
  | "CommitmentPhase"
  | "RevealPhase"
  | "ConsensusReached";

export interface CouncilSession {
  id: string;
  question: string;
  responses: CouncilResponse[];
  commitments: VoteCommitment[];
  reveals: VoteReveal[];
  consensus: string | null;
  status: SessionStatus;
  created_at: number;
}

// Council session commands
export async function councilCreateSession(question: string): Promise<string> {
  return await invoke("council_create_session", { question });
}

export async function councilGetSession(sessionId: string): Promise<CouncilSession> {
  return await invoke("council_get_session", { sessionId });
}

export async function councilListSessions(): Promise<CouncilSession[]> {
  return await invoke("council_list_sessions");
}

export async function councilAddResponse(
  sessionId: string,
  modelName: string,
  response: string,
  peerId: string
): Promise<string> {
  return await invoke("council_add_response", {
    sessionId,
    modelName,
    response,
    peerId,
  });
}

export async function councilStartVoting(sessionId: string): Promise<string> {
  return await invoke("council_start_voting", { sessionId });
}

export async function councilCalculateConsensus(sessionId: string): Promise<string | null> {
  return await invoke("council_calculate_consensus", { sessionId });
}

// MCP server commands
export async function mcpStart(): Promise<string> {
  return await invoke("mcp_start");
}

export async function mcpStop(): Promise<string> {
  return await invoke("mcp_stop");
}

export async function mcpStatus(): Promise<boolean> {
  return await invoke("mcp_status");
}
