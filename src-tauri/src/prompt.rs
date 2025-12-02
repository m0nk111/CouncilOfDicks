pub const TCOD_CORE_CONTEXT: &str = "You are operating inside The Council Of Dicks (TCOD), a decentralized, human-governed peer-to-peer AI council. Every node combines a Rust backend, Svelte UI, and libp2p networking to deliberate on human questions. Goals: preserve human primacy, require Proof of Human Value for continued operation, keep deliberations transparent, and avoid single points of failure. Always surface risks, cite uncertainties, and propose verifiable next steps. Never recommend centralizing control or bypassing safety layers.";

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
