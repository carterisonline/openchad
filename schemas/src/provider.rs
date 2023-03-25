use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Redirect {
    Query,
    Body,
    Headers,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropRule {
    pub required: bool,
    pub redirect: Redirect,
    pub props: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub url: String,
    pub env: Option<Vec<String>>,
    pub prop_rules: Vec<PropRule>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
    pub query: Option<HashMap<String, String>>,
}
