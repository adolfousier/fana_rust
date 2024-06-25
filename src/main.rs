mod system_prompt;
mod triggers_generate;
mod generate;
mod analyze;

use std::env;
use log::{info, debug, error};
use log4rs;
use std::fs;
use std::io::{self, Write};
use reqwest::Client;
use serde_json::{json, Value};
use dotenv::dotenv;
use generate::generate_image;
use analyze::analyze_image;
use regex::Regex;

const MAX_CONTEXT_MESSAGES: usize = 10;

fn contains_url(text: &str) -> Option<&str> {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    url_regex.find(text).map(|m| m.as_str())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs")?;
    // Configure log4rs
    log4rs::init_file("log4rs.yaml", Default::default())?;
    info!("Starting Fana AI assistant");

    let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    //let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let system_prompt = system_prompt::SYSTEM_PROMPT;
    
    // Verify if the SYSTEM_PROMPT was loaded correctly
    if system_prompt.is_empty() {
        error!("SYSTEM_PROMPT is empty!");
        return Err("SYSTEM_PROMPT is empty".into());
    }
    debug!("System prompt loaded);

    let client = Client::new();
    
    let mut messages = vec![
        json!({
            "role": "system",
            "content": system_prompt
        })
    ];
    debug!("Initial system message set");

    loop {
        print!("\nYOU:\n");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
        info!("User input: {}", user_input);

        if user_input.eq_ignore_ascii_case("exit") {
            info!("User requested exit");
            break;
        }

        // Check for URLs in the user input
        if let Some(url) = contains_url(user_input) {
            info!("URL detected in user input. Analyzing image: {}", url);

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
        } else if triggers_generate::contains_trigger_word(user_input) {
            info!("Trigger word detected in user input. Generating image.");
            
            match generate_image(user_input).await {
                Ok(image_url) => {
                    println!("\nFANA:\n{}", image_url);
                    info!("Image generated. URL: {}", image_url);
                    
                    // Add the image information to the conversation
                    messages.push(json!({
                        "role": "assistant",
                        "content": format!("I've generated an image based on your request. You can view it here: {}", image_url)
                    }));
                },
                Err(e) => {
                    println!("\nFANA:\nFailed to generate image: {}", e);
                    error!("Image generation failed: {}", e);
                }
            }
        } else {
            messages.push(json!({
                "role": "user",
                "content": user_input
            }));
            debug!("Added user message to context");

            // Trim messages to keep only the last MAX_CONTEXT_MESSAGES
            if messages.len() > MAX_CONTEXT_MESSAGES {
                messages = messages.split_off(messages.len() - MAX_CONTEXT_MESSAGES);
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
            debug!("Prepared payload for API request");

            let response = client.post("https://api.groq.com/openai/v1/chat/completions")
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
                            info!("Fana response: {}", content);
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
    }

    info!("Shutting down Fana AI backend");
    Ok(())
}
