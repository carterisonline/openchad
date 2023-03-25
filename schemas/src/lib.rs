pub mod botconfig;
pub mod chat;
pub mod search;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategorizeBody {
    pub message: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategorizeResponse {
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatBody {
    pub message: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryBody {
    pub message: String,
    pub user: String,
}

#[cfg(test)]
pub mod tests {
    use architectury::coreutils::*;
    use architectury::prelude::*;

    use crate::botconfig::BotConfig;
    #[test]
    fn parse_bot_json() -> Result<()> {
        architectury::init();
        let parsed_cfg = serde_json::from_str::<BotConfig>(&cat("../bot.json")?);
        dbg!(&parsed_cfg);
        assert!(parsed_cfg.is_ok());

        Ok(())
    }
}
