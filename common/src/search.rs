use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub snippet: String,
    pub url: String,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchPages {
    pub value: Vec<SearchResult>,

    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    #[serde(rename = "_type")]
    _type: String,
    #[serde(rename = "webPages")]
    pub web_pages: SearchPages,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchQueryOptions {
    pub answer_count: Option<String>,
    pub cc: Option<String>,
    pub count: Option<String>,
    pub freshness: Option<String>,
    pub mkt: Option<String>,
    pub offset: Option<String>,
    pub promote: Option<String>,
    pub response_filter: Option<String>,
    pub safe_search: Option<String>,
    pub set_lang: Option<String>,
    pub text_decorations: Option<String>,
    pub text_format: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub q: String,
    #[serde(flatten)]
    pub options: SearchQueryOptions,
}
