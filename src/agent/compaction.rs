use crate::providers::types::{Message, Role};

pub struct ContextCompactor;

impl ContextCompactor {
    /// Prune or compact message history if total estimated characters exceed budget
    pub fn compact_history(messages: &mut Vec<Message>, max_chars: usize) -> bool {
        let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        if total_chars <= max_chars || messages.len() <= 4 {
            return false;
        }

        // Keep system message (first) and latest 3 messages
        let system_msg = if !messages.is_empty() && messages[0].role == Role::System {
            Some(messages[0].clone())
        } else {
            None
        };

        let split_point = messages.len().saturating_sub(4);
        let middle_messages = &messages[1..split_point];

        let mut summary_points = Vec::new();
        for m in middle_messages {
            let role_name = match m.role {
                Role::User => "User asked",
                Role::Assistant => "Agent performed",
                Role::Tool => "Tool output",
                Role::System => "System",
            };
            let preview = if m.content.len() > 120 {
                format!("{}...", &m.content[..120].replace('\n', " "))
            } else {
                m.content.replace('\n', " ")
            };
            summary_points.push(format!("- {}: {}", role_name, preview));
        }

        let summary_text = format!(
            "[Context Compaction: {} intermediate conversation turns summarized below]\n{}",
            middle_messages.len(),
            summary_points.join("\n")
        );

        let latest_messages: Vec<Message> = messages[split_point..].to_vec();

        messages.clear();
        if let Some(sys) = system_msg {
            messages.push(sys);
        }
        messages.push(Message::user(summary_text));
        messages.extend(latest_messages);

        true
    }
}
