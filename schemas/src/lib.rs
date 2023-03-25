pub mod botconfig;
pub mod chat;
pub mod provider;
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
    use crate::provider::Provider;

    #[test]
    fn parse_bot_json() -> Result<()> {
        let parsed_cfg = serde_json::from_str::<BotConfig>(&cat("../bot.json")?);
        dbg!(&parsed_cfg);
        assert!(parsed_cfg.is_ok());

        Ok(())
    }

    #[test]
    fn generate_bot_schema() -> Result<()> {
        let schema = schemars::schema_for!(BotConfig);
        redirect(
            "../bot.schema.json",
            &serde_json::to_string_pretty(&schema)?,
        )?;

        Ok(())
    }

    #[test]
    fn parse_provider_json() -> Result<()> {
        let parsed_cfg = serde_json::from_str::<Provider>(&cat("../providers/bing.json")?);
        dbg!(&parsed_cfg);
        assert!(parsed_cfg.is_ok());

        Ok(())
    }

    #[test]
    fn generate_provider_schema() -> Result<()> {
        let schema = schemars::schema_for!(Provider);
        redirect(
            "../providers/providers.schema.json",
            &serde_json::to_string_pretty(&schema)?,
        )?;

        Ok(())
    }
}
