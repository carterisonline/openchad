use std::env::var;
use std::sync::Arc;

use architectury::prelude::*;
use futures::prelude::*;
use openchad_schemas::botconfig::BotConfig;
use openchad_schemas::chat::{ChatMessage, ChatResponseStream};
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

const CHAT_CHUNKS: usize = 25;

pub async fn chat_request(
    header: &str,
    message: String,
    history: &[ChatMessage],
    config: Arc<BotConfig>,
) -> Result<impl Stream<Item = Result<String, std::io::Error>>> {
    let input = [
        history
            .get(history.len().saturating_sub(config.message_history)..)
            .unwrap_or_default(),
        &[
            ChatMessage {
                role: "system".into(),
                content: header.into(),
            },
            ChatMessage {
                role: "user".into(),
                content: message,
            },
        ],
    ]
    .concat();

    let est_tokens = estimate_tokens(
        &input
            .iter()
            .map(|m| &m.content)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n"),
    );

    warn!(
        "Sending {} tokens (~{}¢)",
        est_tokens,
        ((est_tokens as f64) / 10.0) * 0.002
    );

    let response = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header(
            "Authorization",
            "Bearer ".to_owned() + &var("OPENAI_API_KEY").unwrap(),
        )
        .json(&json!({
            "model": "gpt-3.5-turbo",
            "messages": input,
            "stream": true
        }))
        .send()
        .await?;

    let stream = response
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

    let mut reader = StreamReader::new(stream);

    Ok(async_stream::try_stream! {
        let mut acc = String::new();
        let mut buf = vec![];
        let mut i = 0;
        loop {
            reader.read_line(&mut acc).await?;

            if acc.trim() == "data: [DONE]" {
                yield buf.join("");
                break;
            }

            if acc.starts_with("data: ") {
                if let Ok(frag) = serde_json::from_str::<ChatResponseStream>(acc[6..].trim()) {
                    if let Some(word) = frag.choices[0].delta.get("content") {
                        i += 1;
                        if i % CHAT_CHUNKS == 0 {
                            buf.push(word.to_string());
                            yield buf.join("");
                            buf.clear();
                        } else {
                            buf.push(word.to_string());
                        }
                    }
                }
            }

            acc.clear();
        }
    })
}

fn estimate_tokens(s: &str) -> usize {
    ((s.len() as f32
        / ((s
            .split_ascii_whitespace()
            .fold(0, |acc, s| acc + s.len() as u32) as f32
            / s.split_ascii_whitespace().count() as f32)
            .ceil()))
        * 1.5)
        .ceil() as usize
}
