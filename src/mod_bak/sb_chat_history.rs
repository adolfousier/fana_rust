use crate::embedding::generate_embedding;
use crate::content_fetcher::find_most_similar_chat_history;
use crate::chat_completion::generate_chat_response;
use log::{info, error};
use reqwest::Client;
use serde_json::Value;
use std::env;
use uuid::Uuid;
use tokio::sync::Mutex;
use std::sync::Arc;
use lazy_static::lazy_static;

static SUPABASE_CHAT_HISTORY_URL: &str = env::var("SUPABASE_CHAT_HISTORY_URL").expect("SUPABASE_CHAT_HISTORY_URL must be set").as_str();
static SUPABASE_CHAT_HISTORY_TABLE_NAME: &str = env::var("SUPABASE_CHAT_HISTORY_TABLE_NAME").expect("SUPABASE_CHAT_HISTORY_TABLE_NAME must be set").as_str();
static SUPABASE_CHAT_HISTORY_BEARER_TOKEN: &str = env::var("SUPABASE_CHAT_HISTORY_BEARER_TOKEN").expect("SUPABASE_CHAT_HISTORY_BEARER_TOKEN must be set").as_str();

static OPENAI_API_KEY: &str = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set").as_str();

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub fn generate_user_and_session_ids() -> (String, String) {
    (Uuid::new_v4().to_string(), Uuid::new_v4().to_string())
}

pub async fn store_chat_history(
    chat_history: Vec<Value>, 
    embeddings: Vec<f32>, 
    user_id: &str, 
    session_id: &str
) -> Result<Value, String> {
    let url = format!("{}/rest/v1/{}", SUPABASE_CHAT_HISTORY_URL, SUPABASE_CHAT_HISTORY_TABLE_NAME);

    let data = serde_json::json!({
        "chat_history": { "data": chat_history },
        "embedding": embeddings,
        "user_id": user_id,
        "session_id": session_id,
        "created_at": "now()"
    });

    let response = CLIENT.post(&url)
        .bearer_auth(SUPABASE_CHAT_HISTORY_BEARER_TOKEN)
        .json(&data)
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                info!("Inserted chat history in Supabase successfully.");
                Ok(res.json().await.unwrap())
            } else {
                error!("Failed to insert chat history in Supabase.");
                Err(format!("Failed to insert chat history: {:?}", res.text().await.unwrap()))
            }
        },
        Err(e) => {
            error!("Error storing chat history: {:?}", e);
            Err(format!("Error storing chat history: {:?}", e))
        }
    }
}

pub async fn retrieve_chat_history(
    user_id: &str, 
    session_id: &str, 
    user_input: Option<&str>, 
    embedding: Option<Vec<f32>>, 
    date: Option<&str>, 
    keywords: Option<Vec<&str>>
) -> Vec<Value> {
    let url = format!("{}/rest/v1/{}", SUPABASE_CHAT_HISTORY_URL, SUPABASE_CHAT_HISTORY_TABLE_NAME);

    let mut query_params = vec![
        ("user_id", user_id),
        ("session_id", session_id),
    ];

    if let Some(date) = date {
        query_params.push(("created_at", date));
    }

    let response = CLIENT.get(&url)
        .bearer_auth(SUPABASE_CHAT_HISTORY_BEARER_TOKEN)
        .query(&query_params)
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let data: Vec<Value> = res.json().await.unwrap();
                if let Some(embedding) = embedding {
                    let most_similar_chat_history = find_most_similar_chat_history(embedding, data, user_input.unwrap_or("")).await;
                    match most_similar_chat_history {
                        Ok(history) => {
                            info!("Chat history retrieved successfully.");
                            history
                        },
                        Err(e) => {
                            error!("No similar chat history found: {}", e);
                            vec![]
                        }
                    }
                } else {
                    info!("Chat history retrieved successfully.");
                    data
                }
            } else {
                error!("Failed to retrieve chat history.");
                vec![]
            }
        },
        Err(e) => {
            error!("Error retrieving chat history: {:?}", e);
            vec![]
        }
    }
}

pub async fn summarize_chat_history(chat_history: Vec<Value>) -> Result<String, String> {
    let messages: Vec<Value> = chat_history.iter()
        .map(|entry| serde_json::json!({ "role": "user", "content": entry["content"] }))
        .collect();

    let summary_response = generate_chat_response(&messages).await;

    match summary_response {
        Ok(summary) => {
            info!("Chat history summarized successfully.");
            Ok(summary)
        },
        Err(e) => {
            error!("Error summarizing chat history: {}", e);
            Err(format!("Error summarizing chat history: {}", e))
        }
    }
}

