use architectury::prelude::*;
use openchad_common::search::*;
use std::env::var;

pub async fn search(query: String, options: SearchQueryOptions) -> Result<SearchResponse> {
    info!("Searching for {query}");
    Ok(reqwest::Client::new()
        .get("https://api.bing.microsoft.com/v7.0/search")
        .query(&SearchQuery { q: query, options })
        .header(
            "Ocp-Apim-Subscription-Key",
            var("OCP_APIM_SUBSCRIPTION_KEY").unwrap(),
        )
        .send()
        .await?
        .json::<SearchResponse>()
        .await?)
}
