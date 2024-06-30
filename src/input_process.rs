// input_process.rs
use crate::triggers_generate;
use crate::dotenv;
use crate::session_manager::SessionManager;
use crate::url_handler::handle_url;
use crate::trigger_handler::handle_trigger;
use crate::context_manager::ContextManager;

use serde_json::Value;
use reqwest::Client;
use log::{info, debug, error};
use serde_json::json;

pub const MAX_CONTEXT_MESSAGES: usize = 10;


pub async fn process_user_input(
    user_input: String,
    session_manager: &mut SessionManager,
    client: &Client,
    groq_api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    info!("Processing user input: {}", user_input);
    debug!("Using Groq API Key: {}", groq_api_key);

    let session_id = session_manager.create_session();
    info!("Session ID: {}", session_id); // Use session_id to identify the session

    let mut context_manager = ContextManager::new("memory");
    match context_manager.load_context().await {
    Ok(_) => {
            // context loaded successfully
        }
        Err(e) => {
            // handle error
            eprintln!("Error loading context: {}", e);
        }
    }

    let user_message = json!({
        "role": "user",
        "content": user_input.clone()
    });
    context_manager.add_message(user_message).await;


    // Process user input
    info!("Processing user input: {}", user_input);

    if let Some(url) = crate::url_handler::contains_url(&user_input) {
        let result = handle_url(url, &mut context_manager).await;
        match context_manager.save_context().await {
            Ok(_) => {
                // handle Ok variant
            }
            Err(e) => {
                // handle Err variant
                error!("Error saving context: {}", e);
            }
        }
        return result;
    } else if triggers_generate::contains_trigger_word(&user_input) {
        let result = handle_trigger(&user_input, &mut context_manager).await;
        match context_manager.save_context().await {
            Ok(_) => {
                // handle Ok variant
            }
            Err(e) => {
                // handle Err variant
                error!("Error saving context: {}", e);
            }
        }
        return result;
    } else {
        let result = process_text_input(&user_input, &mut context_manager, client, groq_api_key).await;
        match context_manager.save_context().await {
            Ok(_) => {
                // handle Ok variant
            }
            Err(e) => {
                // handle Err variant
                error!("Error saving context: {}", e);
            }
        }
        return result;
    }
}

async fn process_text_input(
    user_input: &str,
    context_manager: &mut ContextManager,
    client: &Client,
    groq_api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("No URL or trigger word detected. Processing text input: {}", user_input);

    context_manager.trim_context(MAX_CONTEXT_MESSAGES).await;

    debug!("Trimmed context messages to {}", MAX_CONTEXT_MESSAGES);

    let payload = json!({
        "model": "mixtral-8x7b-32768",
        "messages": context_manager.get_context(),
        "temperature": 0.5,
        "max_tokens": 4000,
        "top_p": 1,
        "stop": null,
        "stream": false
    });
    debug!("Prepared payload for API request: {:?}", payload);

    // Log the request details to ensure they are correct
    debug!("Sending request to Groq API with key: {}", groq_api_key);

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
                            context_manager.add_message(json!({
                                "role": "assistant",
                                "content": content
                            })).await;
                            debug!("Added assistant message to context");
                            // Log token usage
                            if let Some(usage) = json["usage"].as_object() {
                                let prompt_tokens = usage.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0);
                                let completion_tokens = usage.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0);
                                let total_tokens = usage.get("total_tokens").and_then(Value::as_u64).unwrap_or(0);
                                info!("Token usage - Prompt tokens: {}, Completion tokens: {}, Total tokens: {}", prompt_tokens, completion_tokens, total_tokens);
                            }
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
