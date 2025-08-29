use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqMessage {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    MultiModal(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    Image { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroqRequest {
    pub model: String,
    pub messages: Vec<GroqMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroqResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct GroqClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl GroqClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: "https://api.groq.com/openai/v1".to_string(),
        }
    }

    pub async fn chat_completion(&self, request: GroqRequest) -> Result<GroqResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Groq API error: {}", error_text));
        }

        let groq_response: GroqResponse = response.json().await?;
        Ok(groq_response)
    }

    pub async fn send_message(
        &self,
        model: &str,
        messages: Vec<GroqMessage>,
        temperature: f32,
    ) -> Result<String> {
        let request = GroqRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens: Some(4096),
            stream: false,
        };

        let response = self.chat_completion(request).await?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response from Groq API"))
        }
    }

    pub fn create_text_message(role: &str, content: &str) -> GroqMessage {
        GroqMessage {
            role: role.to_string(),
            content: MessageContent::Text(content.to_string()),
        }
    }

    pub fn create_image_message(role: &str, text: &str, image_data: &str) -> GroqMessage {
        GroqMessage {
            role: role.to_string(),
            content: MessageContent::MultiModal(vec![
                ContentPart::Text {
                    text: text.to_string(),
                },
                ContentPart::Image {
                    image_url: ImageUrl {
                        url: format!("data:image/png;base64,{}", image_data),
                    },
                },
            ]),
        }
    }
}