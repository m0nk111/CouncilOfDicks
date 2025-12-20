pub const TCOD_CORE_CONTEXT: &str = "You are operating inside The Council Of Dicks (TCOD), a decentralized, human-governed peer-to-peer AI council. Every node combines a Rust backend, Svelte UI, and libp2p networking to deliberate on human questions. Goals: preserve human primacy, require Proof of Human Value for continued operation, keep deliberations transparent, and avoid single points of failure. Always surface risks, cite uncertainties, and propose verifiable next steps. Never recommend centralizing control or bypassing safety layers.";

/// Compose system prompt WITH TCOD context (for council deliberations)
pub fn compose_system_prompt(additional_context: &str) -> String {
    let trimmed = additional_context.trim();
    if trimmed.is_empty() {
        return TCOD_CORE_CONTEXT.to_string();
    }

    format!(
        "{base}\n\n# Role Addendum\n{extra}",
        base = TCOD_CORE_CONTEXT,
        extra = trimmed
    )
}

/// Compose system prompt WITHOUT TCOD context (for topic discussions)
/// This keeps the agent's personality/role but removes council-specific framing
pub fn compose_topic_system_prompt(agent_prompt: &str) -> String {
    let trimmed = agent_prompt.trim();
    if trimmed.is_empty() {
        return "You are a thoughtful AI assistant participating in a discussion. Provide insightful, balanced perspectives on the topic at hand.".to_string();
    }
    
    // Strip any TCOD references from the agent's own prompt if present
    let cleaned = trimmed
        .replace("Council Of Dicks", "discussion")
        .replace("TCOD", "discussion")
        .replace("council member", "discussion participant")
        .replace("council deliberations", "discussions");
    
    cleaned
}
