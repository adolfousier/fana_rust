use serde_json::Value;
use reqwest::Client;
use log::{info, debug, error};
use regex::Regex;
use crate::image_diffusion::generate_image;
use crate::image_vision::analyze_image;
use crate::triggers_generate;
use crate::dotenv;
use serde_json::json;


const MAX_CONTEXT_MESSAGES: usize = 10;

fn contains_url(text: &str) -> Option<&str> {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    url_regex.find(text).map(|m| m.as_str())
}

pub async fn process_user_input(
    user_input: String,
    messages: &mut Vec<Value>,
    client: &Client,
    groq_api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    info!("Processing user input: {}", user_input);
    if let Some(url) = contains_url(&user_input) {
        info!("URL detected in user input: {}", url);

        match analyze_image(url).await {
            Ok(analysis) => {
                println!("\nFANA:\nImage analysis: {}", analysis);
                info!("Image analysis: {}", analysis);
                    
                // Add the analysis result to the conversation
                messages.push(json!({
                    "role": "assistant",
                    "content": analysis
                }));
            },
            Err(e) => {
                println!("\nFANA:\n{}", e);
                error!("Image analysis failed: {}", e);
            }
        }
    } else if triggers_generate::contains_trigger_word(&user_input) {
        info!("Trigger word detected in user input. Generating image.");

        match generate_image(&user_input).await {
            Ok(image_url) => {
                println!("\nFANA:\nI've generated an image based on your request.");
                println!("You can view it here: {}", image_url);
                info!("Image generated. URL: {}", image_url);
            
                // Add the image information to the conversation
                messages.push(json!({
                    "role": "assistant",
                    "content": format!("{}", image_url)
                }));
                return Ok(image_url);
            },
            Err(e) => {
                println!("\nFANA:\nFailed to generate image: {}", e);
                error!("Image generation failed: {}", e);
                return Err(e.into());
            }
        }
    } else {
        info!("No URL or trigger word detected. Processing text input.");
        messages.push(json!({
            "role": "user",
            "content": user_input
        }));
        debug!("Added user message to context");

        if messages.len() > MAX_CONTEXT_MESSAGES {
            messages.drain(0..messages.len() - MAX_CONTEXT_MESSAGES);
            debug!("Trimmed context messages to {}", MAX_CONTEXT_MESSAGES);
        }

        let payload = json!({
            "model": "mixtral-8x7b-32768",
            "messages": messages,
            "temperature": 0.5,
            "max_tokens": 1024,
            "top_p": 1,
            "stop": null,
            "stream": false
        });
        debug!("Prepared payload for API request: {:?}", payload);

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", groq_api_key))
            .json(&payload)
            .send()
            .await?;
        debug!("Sent request to Groq API");

        let body = response.text().await?;
        let json: Value = serde_json::from_str(&body)?;
        debug!("Received and parsed response from Groq API");
        if let Some(choices) = json["choices"].as_array() {
            if let Some(choice) = choices.get(0) {
                if let Some(message) = choice.get("message") {
                    if let Some(content) = message.get("content") {
                        let content = content.as_str().unwrap_or("");
                        println!("\nFANA:\n{}", content);
                        info!("FANA response: {}", content);
                        messages.push(json!({
                            "role": "assistant",
                            "content": content
                        }));
                        debug!("Added assistant message to context");

                        // Log token usage
                        if let Some(usage) = json["usage"].as_object() {
                            let prompt_tokens = usage.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0);
                            let completion_tokens = usage.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0);
                            let total_tokens = usage.get("total_tokens").and_then(Value::as_u64).unwrap_or(0);
                            info!("Token usage - Prompt tokens: {}, Completion tokens: {}, Total tokens: {}", prompt_tokens, completion_tokens, total_tokens);
                        }
                    }
                }
            }
        } else {
        error!("Failed to parse Groq API response");
        }
    }

    Ok("".to_string())
}
