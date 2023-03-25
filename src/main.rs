use std::collections::HashMap;
use std::env::var;
use std::process::exit;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use architectury::coreutils::cat;
use architectury::prelude::*;
use once_cell::sync::Lazy;
use openchad_schemas::botconfig::BotConfig;
use openchad_schemas::{CategorizeBody, CategorizeResponse, ChatBody, HistoryBody};
use reqwest_streams::JsonStreamResponse;
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::{async_trait, prelude::*};
use tap::Tap;
use tokio::spawn;

static mut EDIT_MAP: Lazy<HashMap<u64, Arc<std::sync::Mutex<String>>>> =
    Lazy::new(|| HashMap::new());
static EDIT_INDEX: AtomicU64 = AtomicU64::new(0);

pub fn read_config() -> Result<BotConfig> {
    Ok(serde_json::from_str(&cat(var("CONFIG_PATH")?)?)?)
}

static CONFIG: Lazy<Arc<BotConfig>> = Lazy::new(|| Arc::new(read_config().unwrap()));

struct Handler;

fn remove_mentions<S: AsRef<str>>(msg: S, context: &Context) -> String {
    msg.as_ref()
        .replace(&format!("@{}", context.cache.current_user().tag()), "")
        .trim()
        .into()
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.mentions_me(&context).await.unwrap() {
            let mut retries = 0;
            'retry: loop {
                if retries > 3 {
                    msg.reply(&context, "After 3 tries, I'm still having trouble connecting to OpenAI's servers right now. Please try again later.")
                        .await
                        .unwrap();
                    return;
                }

                let user = &msg.author.name;
                let waiting_reaction_handle = msg.react(&context, '💭').await.unwrap();

                let content = msg.content_safe(&context);
                let content = remove_mentions(&content, &context);

                let CategorizeResponse { category } = if retries == 3 {
                    CategorizeResponse {
                        category: CONFIG.fallback_endpoint.clone(),
                    }
                } else {
                    info!(
                        "GET http://{}/categorize user={}",
                        var("API_URL").unwrap(),
                        user
                    );
                    if let Ok(response) = reqwest::Client::new()
                        .get(&format!("http://{}/categorize", var("API_URL").unwrap()))
                        .timeout(Duration::from_secs(2))
                        .json(&CategorizeBody {
                            message: content.clone(),
                            user: user.clone(),
                        })
                        .send()
                        .await
                    {
                        response.json().await.unwrap_or(CategorizeResponse {
                            category: CONFIG.fallback_endpoint.clone(),
                        })
                    } else {
                        CategorizeResponse {
                            category: CONFIG.fallback_endpoint.clone(),
                        }
                    }
                };

                waiting_reaction_handle.delete(&context).await.unwrap();

                let (ep_url, ep) = CONFIG
                    .endpoints
                    .iter()
                    .filter(|(_, e)| e.id == category)
                    .next()
                    .unwrap();

                let category_reaction_handle = msg.react(&context, ep.icon).await.unwrap();

                let typing_handle = context.http.start_typing(msg.channel_id.0).unwrap();

                let content = format!("{}: {content}", user);

                let stream = if let Ok(response) = reqwest::Client::new()
                    .get(
                        &format!("http://{}{}", var("API_URL").unwrap(), ep_url)
                            .tap(|s| info!("GET {s} user={}", user)),
                    )
                    .timeout(Duration::from_secs(20))
                    .json(&ChatBody {
                        message: content,
                        user: user.clone(),
                    })
                    .send()
                    .await
                {
                    response.json_nl_stream::<String>(1024)
                    // if let Ok(j) = response.into_json() {
                    //     j
                    // } else {
                    //     category_reaction_handle.delete(&context).await.unwrap();
                    //     retries += 1;
                    //     continue 'retry;
                    // }
                } else {
                    category_reaction_handle.delete(&context).await.unwrap();
                    retries += 1;
                    continue 'retry;
                };

                let mut reply_handle = msg.reply(&context, String::from("...")).await.unwrap();
                let edit_index = EDIT_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                stream
                    .for_each(|content| async {
                        if let Ok(content) = content {
                            let msg_contents = unsafe { EDIT_MAP.entry(edit_index).or_default() };

                            {
                                let mut msg_contents = msg_contents.lock().unwrap();
                                msg_contents.push_str(&content);
                            }

                            reply_handle
                                .clone()
                                .edit(&context, |m| m.content(msg_contents.lock().unwrap()))
                                .await
                                .unwrap();
                        }
                    })
                    .await;
                {
                    let msg_contents = unsafe { EDIT_MAP.remove(&edit_index).unwrap() };
                    let m = Arc::try_unwrap(msg_contents).unwrap().into_inner().unwrap();

                    info!(
                        "POST http://{}/history user={}",
                        var("API_URL").unwrap(),
                        user
                    );
                    reqwest::Client::new()
                        .post(format!("http://{}/history", var("API_URL").unwrap()))
                        .json(&HistoryBody {
                            message: m,
                            user: user.clone(),
                        })
                        .send()
                        .await
                        .unwrap();
                }

                reply_handle.suppress_embeds(&context).await.unwrap();

                typing_handle.stop().unwrap();

                category_reaction_handle.delete(&context).await.unwrap();

                break;
            }
        }
    }

    async fn ready(&self, context: Context, ready: Ready) {
        context
            .set_activity(Activity::watching("for mentions"))
            .await;
        info!("Successfully logged in as {}", ready.user.tag());
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    architectury::init();

    let token = var("DISCORD_TOKEN")?;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MESSAGE_TYPING;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        exit(0);
    });

    if let Err(e) = client.start().await {
        error!("Client error: {e}");
    }

    Ok(())
}
