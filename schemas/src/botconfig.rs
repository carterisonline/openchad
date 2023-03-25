use std::collections::HashMap;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! config {
    ($($id: ident: $type: ty),+) => {
        #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        pub struct BotConfig {
            pub endpoints: HashMap<String, ConfigEndpoint>,
            $(pub $id: $type),+
        }

        #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        pub struct BotConfigHeadless {
            pub endpoints: Vec<ConfigEndpoint>,
            $(pub $id: $type),+
        }

        impl From<Arc<BotConfig>> for BotConfigHeadless {
            fn from(config: Arc<BotConfig>) -> Self {
                BotConfigHeadless {
                    $($id: config.$id.clone()),+,
                    endpoints: config
                        .endpoints
                        .clone()
                        .into_iter()
                        .map(|(_, v)| v)
                        .collect(),
                }
            }
        }

    };
}

pub type Transform = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEndpoint {
    pub task: String, // Template auto pointer
    pub categorization: String,
    pub designation: String,
    pub id: String,
    pub icon: char,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub args: Option<Vec<String>>,
    pub prompt: Vec<String>,          // Template
    pub transform: Option<Transform>, // Template
    pub footer: Option<String>,       // Template
}

type ConfigMacro = HashMap<String, HashMap<String, String>>;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigProvider {
    pub args: Option<Vec<String>>,
    pub provider: String,
    pub props: HashMap<String, String>,
    pub transform: Option<Transform>,
}

config! {
    fallback_endpoint: String,
    props: HashMap<String, String>,
    responses: HashMap<String, ConfigResponse>,
    macros: HashMap<String, ConfigMacro>,
    providers: HashMap<String, ConfigProvider>,
    help_prompt: Vec<String>,       // Template
    categorize_prompt: Vec<String>, // Template
    message_history: usize
}
