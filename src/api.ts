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

// Chat types
export type ChannelType = "general" | "human" | "knowledge" | "vote";
export type AuthorType = "human" | "ai" | "system";

export interface Reaction {
  emoji: string;
  author: string;
  timestamp: string;
}

export interface ChatMessage {
  id: string;
  channel: ChannelType;
  author: string;
  author_type: AuthorType;
  content: string;
  timestamp: string;
  signature?: string;
  reply_to?: string;
  reactions: Reaction[];
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

// Provider management types
export type ProviderType = "ollama" | "openai" | "anthropic" | "localembeddings";

export interface ProviderConfig {
  id: string;
  username: string;
  display_name: string;
  provider_type: ProviderType;
  enabled: boolean;
  priority: number;
  config: ProviderSpecificConfig;
}

export type ProviderSpecificConfig =
  | {
      type: "Ollama";
      base_url: string;
      default_model: string;
      embedding_model: string;
      timeout_seconds: number;
    }
  | {
      type: "OpenAI";
      api_key: string;
      base_url?: string;
      organization?: string;
      default_model: string;
    }
  | {
      type: "Anthropic";
      api_key: string;
      default_model: string;
      version: string;
    }
  | {
      type: "LocalEmbeddings";
      model_path?: string;
    };

export interface ProviderHealth {
  healthy: boolean;
  latency_ms?: number;
  error?: string;
}

// Chat commands
export async function chatSendMessage(
  channel: ChannelType,
  author: string,
  authorType: AuthorType,
  content: string,
  signature?: string
): Promise<string> {
  return await invoke("chat_send_message", {
    channel,
    author,
    authorType,
    content,
    signature,
  });
}

export async function chatGetMessages(
  channel: ChannelType,
  limit: number = 50,
  offset: number = 0
): Promise<ChatMessage[]> {
  return await invoke("chat_get_messages", { channel, limit, offset });
}

export async function chatAddReaction(
  channel: ChannelType,
  messageId: string,
  emoji: string,
  author: string
): Promise<void> {
  return await invoke("chat_add_reaction", {
    channel,
    messageId,
    emoji,
    author,
  });
}

export async function chatGetMessageCount(
  channel: ChannelType
): Promise<number> {
  return await invoke("chat_get_message_count", { channel });
}

// Provider management commands
export async function providerAdd(config: ProviderConfig): Promise<string> {
  return await invoke("provider_add", { config });
}

export async function providerList(): Promise<ProviderConfig[]> {
  return await invoke("provider_list");
}

export async function providerRemove(id: string): Promise<boolean> {
  return await invoke("provider_remove", { id });
}

export async function providerTestConnection(id: string): Promise<ProviderHealth> {
  return await invoke("provider_test_connection", { id });
}

export async function providerSetDefault(providerId: string, purpose: "generation" | "embedding"): Promise<void> {
  return await invoke("provider_set_default", { providerId, purpose });
}

export async function providerGenerateUsername(modelName: string, providerName: string): Promise<string> {
  return await invoke("provider_generate_username", { modelName, providerName });
}
