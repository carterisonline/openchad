use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::search::SearchQueryOptions;

pub type Transform = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEndpoint {
    pub task: String, // Template auto pointer
    pub categorization: String,
    pub designation: String,
    pub id: String,
    pub icon: char,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub input: Option<String>,        // Template
    pub prompt: Vec<String>,          // Template
    pub transform: Option<Transform>, // Template
    pub footer: Option<String>,       // Template
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMacro {
    pub tasks: Vec<String>, // Template auto pointer
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSearchQuery {
    pub input: Option<String>,        // Template
    pub output: String,               // Template
    pub transform: Option<Transform>, // Template
    #[serde(flatten)]
    pub query_params: SearchQueryOptions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotConfigHeadless {
    pub fallback_endpoint: String,
    pub props: HashMap<String, String>,
    pub endpoints: Vec<ConfigEndpoint>,
    pub responses: HashMap<String, ConfigResponse>,
    pub macros: HashMap<String, ConfigMacro>,
    pub search_queries: HashMap<String, ConfigSearchQuery>,
    pub help_prompt: Vec<String>,       // Template
    pub categorize_prompt: Vec<String>, // Template
    pub message_history: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotConfig {
    pub fallback_endpoint: String,
    pub props: HashMap<String, String>,
    pub endpoints: HashMap<String, ConfigEndpoint>,
    pub responses: HashMap<String, ConfigResponse>,
    pub macros: HashMap<String, ConfigMacro>,
    pub search_queries: HashMap<String, ConfigSearchQuery>,
    pub help_prompt: Vec<String>,       // Template
    pub categorize_prompt: Vec<String>, // Template
    pub message_history: usize,
}

impl From<Arc<BotConfig>> for BotConfigHeadless {
    fn from(config: Arc<BotConfig>) -> Self {
        BotConfigHeadless {
            fallback_endpoint: config.fallback_endpoint.clone(),
            props: config.props.clone(),
            endpoints: config
                .endpoints
                .clone()
                .into_iter()
                .map(|(_, v)| v)
                .collect(),
            responses: config.responses.clone(),
            macros: config.macros.clone(),
            search_queries: config.search_queries.clone(),
            help_prompt: config.help_prompt.clone(),
            categorize_prompt: config.categorize_prompt.clone(),
            message_history: config.message_history,
        }
    }
}
