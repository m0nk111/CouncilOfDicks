import { apiCall, isTauriEnvironment } from "./api-adapter";

// Lazy-load Tauri invoke so web builds never touch the global
type TauriInvoke = typeof import("@tauri-apps/api/core").invoke;
let cachedInvoke: TauriInvoke | null = null;

async function tauriInvoke<T>(command: string, args?: Record<string, any>): Promise<T> {
  if (!isTauriEnvironment()) {
    throw new Error("Tauri API not available in web mode");
  }

  if (!cachedInvoke) {
    const { invoke } = await import("@tauri-apps/api/core");
    cachedInvoke = invoke;
  }

  return cachedInvoke<T>(command, args || {});
}

// Re-export for convenience
export { isTauriEnvironment };

// API base URL for web mode
const API_BASE_URL = window.location.hostname === "localhost" 
  ? "http://localhost:8080" 
  : `http://${window.location.hostname}:8080`;

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

// Verdict types
export interface Verdict {
  id: string;
  session_id: string;
  question: string;
  verdict: string;
  reasoning: string;
  dissent: string | null;
  confidence: number;
  model_votes: Record<string, string>;
  created_at: string;
}

export async function verdictListRecent(limit: number = 10): Promise<Verdict[]> {
  return await apiCall<Verdict[]>("verdict_list_recent", { limit });
}

export async function verdictGet(verdictId: string): Promise<Verdict> {
  return await apiCall<Verdict>("verdict_get", { verdict_id: verdictId });
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
    return await tauriInvoke("set_debug", { enabled });
  } else {
    // Web mode: not implemented yet (TODO: add to HTTP API)
    console.warn("setDebug not available in web mode");
  }
}

export async function getMetrics(): Promise<PerformanceMetrics> {
  return await tauriInvoke("get_metrics");
}

export async function p2pStart(): Promise<string> {
  return await tauriInvoke("p2p_start");
}

export async function p2pStop(): Promise<string> {
  return await tauriInvoke("p2p_stop");
}

export async function p2pStatus(): Promise<NetworkStatus> {
  return await tauriInvoke("p2p_status");
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
  return await apiCall(
    "council_create_session",
    "POST /api/council/create",
    { question, agent_ids: [] }
  );
}

export async function councilCreateSessionWithAgents(
  question: string,
  agentIds: string[]
): Promise<string> {
  return await apiCall(
    "council_create_session_with_agents",
    "POST /api/council/create",
    { question, agent_ids: agentIds }
  );
}

export async function councilGetSession(sessionId: string): Promise<CouncilSession> {
  return await apiCall(
    "council_get_session",
    "POST /api/council/session",
    { sessionId }
  );
}

export async function councilListSessions(): Promise<{sessions: CouncilSession[]}> {
  return await apiCall(
    "council_list_sessions",
    "GET /api/council/sessions"
  );
}

export async function councilAddResponse(
  sessionId: string,
  modelName: string,
  response: string,
  peerId: string
): Promise<string> {
  return await tauriInvoke("council_add_response", {
    sessionId,
    modelName,
    response,
    peerId,
  });
}

export async function councilStartVoting(sessionId: string): Promise<string> {
  return await tauriInvoke("council_start_voting", { sessionId });
}

export async function councilCalculateConsensus(sessionId: string): Promise<string | null> {
  return await tauriInvoke("council_calculate_consensus", { sessionId });
}

// MCP server commands
export async function mcpStart(): Promise<string> {
  return await tauriInvoke("mcp_start");
}

export async function mcpStop(): Promise<string> {
  return await tauriInvoke("mcp_stop");
}

export async function mcpStatus(): Promise<boolean> {
  return await tauriInvoke("mcp_status");
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
    return await tauriInvoke("chat_send_message", {
      channel,
      author,
      authorType,
      content,
      signature,
    });
  } else {
    // Web mode: use HTTP API
    const response = await fetch(`${API_BASE_URL}/api/chat/send`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        channel,
        author,
        author_type: authorType,
        content,
        signature,
      }),
    });
    const result = await response.json();
    if (!result.success) {
      throw new Error(result.error || "Failed to send message");
    }
    return result.data;
  }
}

export async function chatGetMessages(
  channel: ChannelType,
  limit: number = 50,
  offset: number = 0
): Promise<ChatMessage[]> {
  if (isTauriEnvironment()) {
    return await tauriInvoke("chat_get_messages", { channel, limit, offset });
  } else {
    // Web mode: use HTTP API
    const response = await fetch(`${API_BASE_URL}/api/chat/messages`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ channel, limit, offset }),
    });
    const result = await response.json();
    if (!result.success) {
      throw new Error(result.error || "Failed to get messages");
    }
    return result.data;
  }
}

export async function chatAddReaction(
  channel: ChannelType,
  messageId: string,
  emoji: string,
  author: string
): Promise<void> {
  return await tauriInvoke("chat_add_reaction", {
    channel,
    messageId,
    emoji,
    author,
  });
}

export async function chatGetMessageCount(
  channel: ChannelType
): Promise<number> {
  return await tauriInvoke("chat_get_message_count", { channel });
}

export async function chatCheckDuplicate(
  question: string
): Promise<DuplicateCheckResult> {
  return await tauriInvoke('chat_check_duplicate', { question });
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
  if (isTauriEnvironment()) {
    return await tauriInvoke('chat_check_rate_limit', { userId });
  }
  console.warn('chatCheckRateLimit: skipping rate limit check in web mode');
  return { allowed: true };
}

export async function chatRecordQuestion(userId: string): Promise<void> {
  if (isTauriEnvironment()) {
    return await tauriInvoke('chat_record_question', { userId });
  }
  console.warn('chatRecordQuestion: web mode no-op');
}

// Spam detection API
export async function chatCheckSpam(userId: string, message: string): Promise<SpamCheckResult> {
  if (isTauriEnvironment()) {
    return await tauriInvoke('chat_check_spam', { userId, message });
  }
  console.warn('chatCheckSpam: skipping spam check in web mode');
  return {
    is_spam: false,
    spam_score: 0,
    spam_level: 'Ok',
    reasons: [],
  };
}

export async function chatRecordMessage(userId: string, message: string): Promise<void> {
  if (isTauriEnvironment()) {
    return await tauriInvoke('chat_record_message', { userId, message });
  }
  console.warn('chatRecordMessage: web mode no-op');
}

// Provider management commands
export async function providerAdd(config: ProviderConfig): Promise<string> {
  return await tauriInvoke("provider_add", { config });
}

export async function providerList(): Promise<ProviderConfig[]> {
  return await tauriInvoke("provider_list");
}

export async function providerRemove(id: string): Promise<boolean> {
  return await tauriInvoke("provider_remove", { id });
}

export async function providerTestConnection(id: string): Promise<ProviderHealth> {
  return await tauriInvoke("provider_test_connection", { id });
}

export async function providerSetDefault(providerId: string, purpose: "generation" | "embedding"): Promise<void> {
  return await tauriInvoke("provider_set_default", { providerId, purpose });
}

export async function providerGenerateUsername(modelName: string, providerName: string): Promise<string> {
  return await tauriInvoke("provider_generate_username", { modelName, providerName });
}

// Agent pool management types
export interface Agent {
  id: string;
  name: string;
  model: string;
  system_prompt: string;
  enabled_tools: string[];
  temperature: number;
  active: boolean;
  metadata: Record<string, string>;
}

export interface Tool {
  name: string;
  description: string;
  parameters: any; // JSON schema
}

// Agent pool commands
export async function agentAdd(
  name: string,
  model: string,
  systemPrompt: string
): Promise<string> {
  return await apiCall("agent_add", "POST /api/agents/create", {
    name,
    model_name: model,
    system_prompt: systemPrompt,
  });
}

export async function agentRemove(agentId: string): Promise<void> {
  return await apiCall("agent_remove", "POST /api/agents/delete", { agent_id: agentId });
}

export async function agentUpdate(agent: Agent): Promise<void> {
  return await apiCall("agent_update", "POST /api/agents/update", { agent });
}

export async function agentList(): Promise<Agent[]> {
  const result = await apiCall<any>("agent_list", "GET /api/agents");

  if (Array.isArray(result)) {
    return result as Agent[];
  }

  if (result && typeof result === "object") {
    if (Array.isArray(result.data)) {
      return result.data as Agent[];
    }

    if (Array.isArray((result as any).agents)) {
      return (result as any).agents as Agent[];
    }
  }

  throw new Error("Unexpected agent list response shape");
}

export async function agentGet(agentId: string): Promise<Agent> {
  return await tauriInvoke("agent_get", { agentId });
}

export async function agentListActive(): Promise<Agent[]> {
  return await tauriInvoke("agent_list_active");
}

export async function agentGetTools(): Promise<Tool[]> {
  return await tauriInvoke("agent_get_tools");
}

// Aliases for backward compatibility
export const agentCreate = agentAdd;
export const agentDelete = agentRemove;
