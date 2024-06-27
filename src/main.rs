// main.rs 
mod system_prompt;
mod triggers_generate;
mod image_diffusion;
mod image_vision;
mod api_auth;
mod api_routes;

use std::env;
use log::{info, debug, error};
use log4rs;
use std::fs;
use std::io::{self, Write};
use reqwest::Client;
use serde_json::{json, Value};
use anyhow::{anyhow};
use actix_web::{App, HttpServer, middleware, web};
use dotenv::dotenv;
use image_diffusion::generate_image;
use image_vision::analyze_image;
use regex::Regex;

const MAX_CONTEXT_MESSAGES: usize = 10;

fn contains_url(text: &str) -> Option<&str> {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    url_regex.find(text).map(|m| m.as_str())
}

async fn process_user_input(
    user_input: String,
    messages: &mut Vec<Value>,
    client: &Client,
    groq_api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Processing user input");
    if let Some(url) = contains_url(&user_input) {
        info!("URL detected in user input: {}", url);

        match analyze_image(url).await {
            Ok(analysis) => {
                info!("Image analysis completed: {}", analysis);
                messages.push(json!({
                    "role": "assistant",
                    "content": analysis
                }));
                return Ok(analysis);
            }
            Err(e) => {
                error!("Image analysis failed: {}", e);
                return Err(e.into());
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
            },
            Err(e) => {
                println!("\nFANA:\nFailed to generate image: {}", e);
                error!("Image generation failed: {}", e);
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
        debug!("Prepared payload for API request");

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

async fn run_interactive_mode(client: Client, groq_api_key: String, system_prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = vec![
        json!({
            "role": "system",
            "content": system_prompt
        })
    ];
    debug!("Initial system message set");

    loop {
        print!("\nYou:\n");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim().to_string();
        info!("User input: {}", user_input);

        if user_input.eq_ignore_ascii_case("exit") {
            info!("User requested exit");
            break;
        }

        if let Err(e) = process_user_input(user_input.clone(), &mut messages, &client, &groq_api_key).await {
            error!("Error processing user input: {}", e);
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs")?;
    // Configure log4rs
    log4rs::init_file("log4rs.yaml", Default::default()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, anyhow!(e)))?;

    info!("Starting Fana AI assistant");

    let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    let system_prompt = system_prompt::SYSTEM_PROMPT.to_string();

    if system_prompt.is_empty() {
        error!("SYSTEM_PROMPT is empty!");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "SYSTEM_PROMPT is empty"));
    }
    debug!("System prompt loaded successfully");

    let client = Client::new();

    // Clone the variables to move them into the thread
    let client_clone = client.clone();
    let groq_api_key_clone = groq_api_key.clone();
    let system_prompt_clone = system_prompt.clone();

    // Spawn a new thread for the interactive console mode
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_interactive_mode(client_clone, groq_api_key_clone, system_prompt_clone).await {
                error!("Error in interactive mode: {}", e);
            }
        });
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(api_auth::ApiKey)
            .configure(api_routes::configure)
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(groq_api_key.clone()))
            .app_data(web::Data::new(system_prompt.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

