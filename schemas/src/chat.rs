use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl From<String> for ChatMessage {
    fn from(value: String) -> Self {
        Self {
            role: "assistant".into(),
            content: value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum FinishReason {
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "stop")]
    #[default]
    Stop,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ChatCategory {
    Conversational,
    InformativeOffline,
    InformativeOnline,
    ImageDescribe,
    ImageTranscribe,
    Link,
    Help,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponseChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: FinishReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub object: String,
    pub created: u64,
    pub choices: Vec<ChatResponseChoice>,
    pub usage: ChatUsage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponseStreamChoice {
    pub delta: HashMap<String, String>,
    pub index: usize,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponseStream {
    pub choices: Vec<ChatResponseStreamChoice>,
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
}
