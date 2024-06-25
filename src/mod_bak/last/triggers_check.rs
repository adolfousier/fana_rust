use std::env;
use log::{info, debug, error};
use log4rs;
use std::fs;
use std::io::{self, Write};
use reqwest::Client;
use serde_json::{json, Value};
use dotenv::dotenv;

const MAX_CONTEXT_MESSAGES: usize = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs")?;

    // Configure log4rs
    log4rs::init_file("log4rs.yaml", Default::default())?;

    info!("Starting Fana AI assistant");

    let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    let client = Client::new();

    let mut messages = vec![
        json!({
            "role": "system",
            "content": "You're Fana AI assistant, an advanced artificial intelligence platform designed to enhance user experiences and streamline various tasks through intelligent automation and contextual understanding."
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
            .header("Authorization", format!("Bearer {}", api_key))
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
                    }
                }
            }
        } else {
            error!("Failed to parse Groq API response");
        }
    }

    info!("Shutting down Fana AI assistant");
    Ok(())
}
