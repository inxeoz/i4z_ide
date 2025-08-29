use crate::api::GroqMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    messages: Vec<GroqMessage>,
    max_history: usize,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_history: 50, // Keep last 50 messages to manage context length
        }
    }

    pub fn add_message(&mut self, message: GroqMessage) {
        self.messages.push(message);
        
        // Trim conversation if it gets too long
        if self.messages.len() > self.max_history {
            // Keep system message (if present) and remove oldest user/assistant messages
            let system_msgs: Vec<GroqMessage> = self.messages
                .iter()
                .filter(|msg| msg.role == "system")
                .cloned()
                .collect();
            
            let other_msgs: Vec<GroqMessage> = self.messages
                .iter()
                .filter(|msg| msg.role != "system")
                .rev()
                .take(self.max_history - system_msgs.len())
                .cloned()
                .collect();
            
            self.messages = system_msgs;
            self.messages.extend(other_msgs.into_iter().rev());
        }
    }

    pub fn get_messages(&self) -> &Vec<GroqMessage> {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn add_system_message(&mut self, content: String) {
        let system_message = GroqMessage {
            role: "system".to_string(),
            content: crate::api::MessageContent::Text(content),
        };
        
        // Insert system message at the beginning
        self.messages.insert(0, system_message);
    }

    pub fn get_last_user_message(&self) -> Option<&GroqMessage> {
        self.messages
            .iter()
            .rev()
            .find(|msg| msg.role == "user")
    }

    pub fn get_last_assistant_message(&self) -> Option<&GroqMessage> {
        self.messages
            .iter()
            .rev()
            .find(|msg| msg.role == "assistant")
    }

    pub fn export_to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn import_from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}