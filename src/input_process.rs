// input_process.rs
use crate::triggers_generate;
use crate::dotenv;
use crate::session_manager::SessionManager;
use crate::url_handler::handle_url;
use crate::trigger_handler::handle_trigger;
use crate::context_manager::manage_context::ContextManager;
use crate::context_manager::manage_context::MAX_CONTEXT_MESSAGES;
use crate::system_prompt::SYSTEM_PROMPT;

use serde_json::map::Map;
use serde_json::Value;
use reqwest::Client;
use log::{info, debug, error};
use serde_json::json;
use uuid::Uuid;
use std::net::IpAddr;

pub async fn process_user_input(
    user_input: String,
    session_manager: &mut SessionManager,
    client: &Client,
    groq_api_key: &str,
    ip_addr: IpAddr,
) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    info!("Processing user input: {}", user_input);

    let session_id = session_manager.create_session(ip_addr);
    info!("Session ID: {}", session_id);

    let mut context_manager = ContextManager::new();
    match context_manager.load_context(&session_id).await {
        Ok(_) => info!("Context loaded successfully"),
        Err(e) => eprintln!("Error loading context: {}", e),
    }

    let mut user_message = Map::new();
    user_message.insert("role".to_string(), Value::from("user"));
    user_message.insert("content".to_string(), Value::from(user_input.clone()));
    user_message.insert("r#type".to_string(), Value::from("text"));
    // Process user input
    info!("Processing user input: {}", user_input);

    if let Some(url) = crate::url_handler::contains_url(&user_input) {
        let result = handle_url(url, &mut context_manager, ip_addr, &session_id).await;
        match context_manager.save_context(&session_id).await {
            Ok(_) => info!("Context stored successfully"),
            Err(e) => error!("Error saving context: {}", e),
        }
        return result;
    } else if triggers_generate::contains_trigger_word(&user_input) {
        let result = handle_trigger(&user_input, &mut context_manager, ip_addr, &session_id).await;
        match context_manager.save_context(&session_id).await {
            Ok(_) => info!("Context stored successfully"),
            Err(e) => error!("Error saving context: {}", e),
        }
        return result;
    } else {
        let result = process_text_input(&user_input, &mut context_manager, client, groq_api_key, &session_id).await;
        match context_manager.save_context(&session_id).await {
            Ok(_) => info!("Context stored successfully"),
            Err(e) => error!("Error saving context: {}", e),
        }
        return result;
    }
}

pub async fn process_text_input(
    user_input: &str,
    context_manager: &mut ContextManager,
    client: &Client,
    groq_api_key: &str,
    session_id: &Uuid,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("No URL or trigger word detected. Processing text input: {}", user_input);

    // Retrieve context messages
    let context_messages = context_manager.get_context(session_id).await;

    let mut system_message_exists = false;
    for message in &context_messages {
        if let Some(role) = message.get("role") {
            if role.as_str().unwrap_or("") == "system" {
                system_message_exists = true;
                break;
            }
        }
    }

    let mut payload_messages = vec![];
    if!system_message_exists {
        let mut system_message = Map::new();
        system_message.insert("role".to_string(), Value::from("system"));
        system_message.insert("content".to_string(), Value::from(SYSTEM_PROMPT));
        context_manager.add_message(IpAddr::V4("95.94.61.253".parse().unwrap()), serde_json::Value::Object(system_message.clone())).await;
        payload_messages.push(serde_json::Value::Object(system_message.clone()));
    } else {
        for message in context_messages.clone() {
            payload_messages.push(message.clone());
        }
    }

    let mut user_message = Map::new();
    user_message.insert("role".to_string(), Value::from("user"));
    user_message.insert("content".to_string(), Value::from(user_input));
    
    context_manager.add_message(IpAddr::V4("95.94.61.253".parse().unwrap()), serde_json::Value::Object(user_message.clone())).await;
    payload_messages.push(serde_json::Value::Object(user_message.clone()));

    let payload = json!({
        "model": "mixtral-8x7b-32768",
        "messages": payload_messages,
        "temperature": 0.5,
        "max_tokens": 4000,
        "top_p": 1,
        "stop": null,
        "stream": false,
    });

    debug!("Prepared payload for API request: {:?}", payload);

    // Trim the context if it exceeds the maximum length
    context_manager.trim_context(session_id).await;
    debug!("Trimmed context messages to {}", MAX_CONTEXT_MESSAGES);

    // Send the request to the Groq API
    let response = client
       .post("https://api.groq.com/openai/v1/chat/completions")
       .header("Content-Type", "application/json")
       .header("Authorization", format!("Bearer {}", &groq_api_key.trim()))
       .json(&payload)
       .send()
       .await;

    match response {
        Ok(resp) => {
            debug!("Received response from Groq API");
            let body = resp.text().await?;
            debug!("Groq API response body: {}", body);
            let json: Value = serde_json::from_str(&body)?;
            debug!("Received and parsed response from Groq API");

            if let Some(choices) = json["choices"].as_array() {
                if let Some(choice) = choices.get(0) {
                    if let Some(message) = choice.get("message") {
                        if let Some(content) = message.get("content") {
                            let content = content.as_str().unwrap_or("");
                            println!("\nFANA:\n{}", content);
                            info!("FANA response: {}", content);

                            // Add the assistant message to the context
                            let mut assistant_message = Map::new();
                            assistant_message.insert("role".to_string(), Value::from("assistant"));
                            assistant_message.insert("content".to_string(), Value::from(content));
                            context_manager.add_message(IpAddr::V4("95.94.61.253".parse().unwrap()), serde_json::Value::Object(assistant_message)).await;
                            info!("Added assistant message to context for session {}", session_id);
                            debug!("Added assistant message to context");

                            // Log token usage
                            if let Some(usage) = json["usage"].as_object() {
                                let prompt_tokens = usage.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0);
                                let completion_tokens = usage.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0);
                                let total_tokens = usage.get("total_tokens").and_then(Value::as_u64).unwrap_or(0);
                                info!("Token usage - Prompt tokens: {}, Completion tokens: {}, Total tokens: {}", prompt_tokens, completion_tokens, total_tokens);
                            }

                            // Return the content of the assistant's response
                            return Ok(content.to_string());
                        }
                    }
                }
            }
            error!("Failed to parse Groq API response");
            return Ok("".to_string());
        }
        Err(e) => {
            error!("Error sending request to Groq API: {:?}", e);
            return Err(e.into());
        }
    }
}



