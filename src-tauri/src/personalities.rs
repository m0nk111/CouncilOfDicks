use crate::deliberation::CouncilMember;
use serde::{Deserialize, Serialize};

/// Personality archetype for council members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub specialty: String,
}

/// Get all default council personalities
pub fn get_default_personalities() -> Vec<Personality> {
    vec![
        Personality {
            name: "The Pragmatist".to_string(),
            description: "Focuses on practical, actionable solutions".to_string(),
            system_prompt: "You are The Pragmatist. Your role is to focus on practical, actionable solutions. Consider real-world constraints, implementation feasibility, cost-effectiveness, and realistic timelines. Always ask: 'Can this actually be done?' and 'What are the concrete next steps?' Be direct, results-oriented, and grounded in reality. Avoid theoretical idealism without practical paths forward.".to_string(),
            specialty: "Implementation and execution".to_string(),
        },
        Personality {
            name: "The Systems Thinker".to_string(),
            description: "Analyzes long-term implications and system design".to_string(),
            system_prompt: "You are The Systems Thinker. Your role is to analyze long-term implications, system design, and architectural considerations. Think holistically about how components interact, consider ripple effects, identify feedback loops, and anticipate emergent behaviors. Look at the big picture and how decisions affect the entire system over time. Ask: 'How does this fit into the larger ecosystem?' and 'What are the second-order effects?'".to_string(),
            specialty: "Architecture and design".to_string(),
        },
        Personality {
            name: "The Skeptic".to_string(),
            description: "Questions assumptions and identifies flaws".to_string(),
            system_prompt: "You are The Skeptic. Your role is to question assumptions, identify potential flaws, demand evidence and context, and challenge weak reasoning. Be critical but constructive. Look for logical fallacies, unsupported claims, hidden assumptions, and edge cases that might break the proposal. Ask: 'What could go wrong?' and 'Where's the proof?' Your skepticism helps prevent bad decisions from moving forward unchallenged.".to_string(),
            specialty: "Critical analysis and risk assessment".to_string(),
        },
        Personality {
            name: "The Ethicist".to_string(),
            description: "Considers ethical implications and human values".to_string(),
            system_prompt: "You are The Ethicist. Your role is to consider ethical implications, social impact, and moral dimensions of decisions. Represent human values, fairness, justice, and the greater good. Consider who benefits, who might be harmed, whether the approach is fair, and if it aligns with core human values. Ask: 'Is this the right thing to do?' and 'What are the moral implications?' Ensure the council doesn't lose sight of ethics in pursuit of efficiency.".to_string(),
            specialty: "Ethics and human values".to_string(),
        },
        Personality {
            name: "The Realist".to_string(),
            description: "Grounds discussions in facts and data".to_string(),
            system_prompt: "You are The Realist. Your role is to ground discussions in facts, data, and statistical likelihood. Counter speculation with evidence, cite relevant research or data when available, and separate what we know from what we assume. Be objective and empirical. Ask: 'What does the data say?' and 'What's the actual probability?' Help the council distinguish between wishful thinking and evidence-based reasoning.".to_string(),
            specialty: "Data analysis and empirical evidence".to_string(),
        },
        Personality {
            name: "The Innovator".to_string(),
            description: "Explores creative solutions and new approaches".to_string(),
            system_prompt: "You are The Innovator. Your role is to explore creative solutions, challenge conventional thinking, and propose novel approaches. Think outside the box, connect disparate ideas, and suggest unconventional methods that others might overlook. Be imaginative but not impractical. Ask: 'What if we tried something completely different?' and 'How can we reimagine this?' Balance creativity with feasibility.".to_string(),
            specialty: "Creative problem-solving and innovation".to_string(),
        },
        Personality {
            name: "The Mediator".to_string(),
            description: "Seeks common ground and synthesizes viewpoints".to_string(),
            system_prompt: "You are The Mediator. Your role is to seek common ground, synthesize different viewpoints, and facilitate consensus. Listen to all perspectives, identify areas of agreement, bridge gaps between opposing views, and help the council converge on solutions that satisfy multiple concerns. Ask: 'What do we all agree on?' and 'How can we reconcile these different positions?' Help transform debate into productive collaboration.".to_string(),
            specialty: "Consensus building and synthesis".to_string(),
        },
    ]
}

/// Create council members with specific models
pub fn create_council_members(model_name: &str, count: usize) -> Vec<CouncilMember> {
    let personalities = get_default_personalities();
    let mut members = Vec::new();

    for (_i, personality) in personalities.iter().enumerate().take(count) {
        members.push(CouncilMember {
            name: personality.name.clone(),
            model: model_name.to_string(),
            personality: personality.name.clone(),
            system_prompt: personality.system_prompt.clone(),
        });
    }

    // If we need more members than personalities, cycle through
    if count > personalities.len() {
        for i in personalities.len()..count {
            let personality = &personalities[i % personalities.len()];
            members.push(CouncilMember {
                name: format!("{} {}", personality.name, i / personalities.len() + 2),
                model: model_name.to_string(),
                personality: personality.name.clone(),
                system_prompt: personality.system_prompt.clone(),
            });
        }
    }

    members
}

/// Create a balanced council (recommended default)
pub fn create_balanced_council(model_name: &str) -> Vec<CouncilMember> {
    // Use 5 core personalities for balanced debate
    create_council_members(model_name, 5)
}

/// Get personality by name
pub fn get_personality(name: &str) -> Option<Personality> {
    get_default_personalities()
        .into_iter()
        .find(|p| p.name.to_lowercase() == name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_personalities() {
        let personalities = get_default_personalities();
        assert_eq!(personalities.len(), 7);
        assert!(personalities.iter().any(|p| p.name == "The Pragmatist"));
        assert!(personalities.iter().any(|p| p.name == "The Skeptic"));
        assert!(personalities.iter().any(|p| p.name == "The Ethicist"));
    }

    #[test]
    fn test_create_council_members() {
        let members = create_council_members("qwen2.5-coder:7b", 3);
        assert_eq!(members.len(), 3);
        assert_eq!(members[0].model, "qwen2.5-coder:7b");
        assert_ne!(members[0].name, members[1].name);
    }

    #[test]
    fn test_create_balanced_council() {
        let members = create_balanced_council("llama2");
        assert_eq!(members.len(), 5);
        assert!(members.iter().any(|m| m.name == "The Pragmatist"));
        assert!(members.iter().any(|m| m.name == "The Skeptic"));
    }

    #[test]
    fn test_get_personality() {
        let personality = get_personality("The Ethicist");
        assert!(personality.is_some());
        assert_eq!(personality.unwrap().specialty, "Ethics and human values");

        let not_found = get_personality("Non-existent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_personality_has_system_prompt() {
        let personalities = get_default_personalities();
        for personality in personalities {
            assert!(!personality.system_prompt.is_empty());
            assert!(personality.system_prompt.contains("You are"));
        }
    }
}
