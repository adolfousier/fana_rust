use log::{info, error};
use crate::modules::logging_setup::setup_logging;
use crate::modules::triggers_check::check_for_trigger_words;
use crate::modules::process_image::process_image_interaction;
use crate::modules::process_text::handle_text_interaction;
use regex::Regex;
use uuid::Uuid;
use serde_json::Value;
use actix_multipart::Multipart;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref OPENAI_API_KEY: String = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
}

pub async fn generate_user_id() -> String {
    Uuid::new_v4().to_string()
}

pub async fn generate_unique_conversation_id() -> String {
    Uuid::new_v4().to_string()
}

pub async fn initialize_context(user_input: &str, chat_history: &Value) {
    // Context initialization logic
}

pub async fn process_url(user_input: &str, chat_history: &mut Value, user_id: &str, session_id: &str, image: Option<Multipart>) -> (Option<String>, Value) {
    let re = Regex::new(r"(https?://\S+)").unwrap();
    if let Some(captures) = re.captures(user_input) {
        if let Some(url) = captures.get(0) {
            let message = user_input.replace(url.as_str(), "").trim().to_string();
            let combined_input = format!("{} {}", message, url.as_str()).trim().to_string();
            info!("Sending request to process_image_interaction module.");
            let (response, updated_chat_history) = process_image_interaction(&combined_input, chat_history, user_id, session_id, image).await;
            if let Some(response) = response {
                return (Some(response), updated_chat_history);
            }
        }
    }
    (None, chat_history.clone())
}

pub async fn process_triggers(
    user_input: &str,
    chat_history: &mut Value,
    user_id: &str,
    session_id: &str,
    image: Option<Multipart>,
) -> (Option<String>, Value) {
    let user_input_lower = user_input.to_lowercase();
    let (triggers_found, triggers) = check_for_trigger_words(&user_input_lower).await;
    if triggers_found {
        let (response, updated_chat_history) = process_image_interaction(user_input, chat_history, user_id, session_id, image).await;
        if let Some(response) = response {
            return (Some(response), updated_chat_history);
        }
    }
    (None, chat_history.clone())
}

pub async fn process_text(
    user_input: &str,
    chat_history: &Value,
    user_id: &str,
    session_id: &str,
) -> (String, Value) {
    handle_text_interaction(user_input, chat_history, user_id, session_id).await
}

pub async fn handle_llm_interaction(
    user_input: &str,
    mut chat_history: Value,
    user_id: Option<String>,
    session_id: Option<String>,
    image: Option<Multipart>,
    from_blob: bool,
) -> (String, Value) {
    info!("Entering Handle LLM Interaction Module");

    initialize_context(user_input, &chat_history).await;

    let user_id = match user_id {
        Some(id) => id,
        None => generate_user_id().await,
    };

    let session_id = match session_id {
        Some(id) => id,
        None => generate_unique_conversation_id().await,
    };

    if from_blob {
        info!("Processing request with handle llm interaction.");
        let (response, updated_chat_history) = process_url(user_input, &mut chat_history, &user_id, &session_id, image.clone()).await;
        if let Some(response) = response {
            return (response, updated_chat_history);
        }
    }

    let (response, updated_chat_history) = process_url(user_input, &mut chat_history, &user_id, &session_id, image.clone()).await;
    if let Some(response) = response {
        return (response, updated_chat_history);
    }

    let (response, updated_chat_history) = process_triggers(user_input, &mut chat_history, &user_id, &session_id, image.clone()).await;
    if let Some(response) = response {
        return (response, updated_chat_history);
    }

    process_text(user_input, &chat_history, &user_id, &session_id).await
}

