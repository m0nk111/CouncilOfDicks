import { invoke } from "@tauri-apps/api/core";
import { apiCall, isTauriEnvironment } from "./api-adapter";

// Re-export for convenience
export { isTauriEnvironment };

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

export interface DuplicateCheckResult {
  is_duplicate: boolean;
  similarity_score: number;
  existing_session_id?: string;
  existing_question?: string;
  existing_verdict?: string;
  asked_at?: string;
}

export async function askCouncil(question: string): Promise<string> {
  return await apiCall<string>("ask_ollama", "POST /api/ollama/ask", { 
    prompt: question,
    model: undefined // Use default model
  }).then(result => {
    // Web mode returns {response: string}, Tauri returns string directly
    return typeof result === 'object' && 'response' in result ? (result as any).response : result;
  });
}

export async function getConfig(): Promise<AppConfig> {
  return await apiCall<AppConfig>("get_config", "GET /api/config");
}

export async function setDebug(enabled: boolean): Promise<void> {
  if (isTauriEnvironment()) {
    return await invoke("set_debug", { enabled });
  } else {
    // Web mode: not implemented yet (TODO: add to HTTP API)
    console.warn("setDebug not available in web mode");
  }
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
  if (isTauriEnvironment()) {
    return await invoke("chat_send_message", {
      channel,
      author,
      authorType,
      content,
      signature,
    });
  } else {
    // Web mode: TODO - add HTTP endpoint
    console.warn("chatSendMessage not yet available in web mode");
    return "message-id-placeholder";
  }
}

export async function chatGetMessages(
  channel: ChannelType,
  limit: number = 50,
  offset: number = 0
): Promise<ChatMessage[]> {
  if (isTauriEnvironment()) {
    return await invoke("chat_get_messages", { channel, limit, offset });
  } else {
    // Web mode: TODO - add HTTP endpoint
    console.warn("chatGetMessages not yet available in web mode");
    return [];
  }
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

export async function chatCheckDuplicate(
  question: string
): Promise<DuplicateCheckResult> {
  return await invoke('chat_check_duplicate', { question });
}

// Rate limiting types
export interface RateLimitResult {
  allowed: boolean;
  reason?: string;
  retry_after_seconds?: number;
}

// Spam detection types
export type SpamLevel = 'Ok' | 'Warning' | 'Cooldown5m' | 'Cooldown1h' | 'Ban24h';

export interface SpamCheckResult {
  is_spam: boolean;
  spam_score: number;
  spam_level: SpamLevel;
  reasons: string[];
  cooldown_seconds?: number;
}

// Rate limiting API
export async function chatCheckRateLimit(userId: string): Promise<RateLimitResult> {
  return await invoke('chat_check_rate_limit', { userId });
}

export async function chatRecordQuestion(userId: string): Promise<void> {
  return await invoke('chat_record_question', { userId });
}

// Spam detection API
export async function chatCheckSpam(userId: string, message: string): Promise<SpamCheckResult> {
  return await invoke('chat_check_spam', { userId, message });
}

export async function chatRecordMessage(userId: string, message: string): Promise<void> {
  return await invoke('chat_record_message', { userId, message });
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
