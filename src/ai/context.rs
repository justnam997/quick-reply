use crate::models::Message;

/// Assembles the memory context payload piped to Claude CLI via stdin.
pub struct ContextAssembler;

impl ContextAssembler {
    pub fn assemble(
        contact_name: &str,
        profile_summary: Option<&str>,
        recent_messages: &[Message],
        relevant_past: &[Message],
    ) -> String {
        let mut sections = Vec::new();

        // Contact profile section
        if let Some(profile) = profile_summary {
            sections.push(format!("[Contact Profile: {contact_name}]\n{profile}"));
        }

        // Relevant past context from semantic search
        if !relevant_past.is_empty() {
            let mut past_section = "[Relevant Past Context]\n".to_string();
            for msg in relevant_past {
                let direction = if msg.is_outgoing { "You" } else { &msg.sender };
                past_section.push_str(&format!("{}: {}\n", direction, msg.content));
            }
            sections.push(past_section);
        }

        // Recent conversation
        let mut recent_section = "[Recent Messages]\n".to_string();
        for msg in recent_messages {
            let direction = if msg.is_outgoing { "You" } else { &msg.sender };
            let time = msg.timestamp.format("%H:%M");
            recent_section.push_str(&format!("[{}] {}: {}\n", time, direction, msg.content));
        }
        sections.push(recent_section);

        sections.join("\n---\n")
    }
}
