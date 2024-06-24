use std::env;
use std::io::{self, Write};
use reqwest::Client;
use serde_json::{json, Value};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from the .env file
    dotenv().ok();
    // Fetch the Claude API key from the environment variable
    let api_key = env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY not set");

    // Create a reqwest client
    let client = Client::new();

    // Initialize the conversation history buffer with a capacity of 5
    let mut buffer: Vec<Value> = Vec::with_capacity(5);
    buffer.push(json!({
        "role": "system",
        "content": "You are a helpful assistant."
    }));

    loop {
        // Prompt the user for input
        print!("You: ");
        io::stdout().flush()?; // Flush the stdout buffer to ensure the prompt is displayed

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        // Exit the loop if the user types "exit"
        if user_input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Add the user's input to the buffer
        buffer.push(json!({
            "role": "user",
            "content": user_input
        }));

        // Ensure the buffer does not exceed 5 messages
        if buffer.len() > 5 {
            buffer.remove(0); // Remove the oldest message
        }

        // Define the JSON payload with the current buffer
        let payload = json!({
            "model": "claude-v1",
            "messages": buffer,
            "temperature": 0.5,
            "max_tokens": 1024,
            "top_p": 1,
            "stop": null,
            "stream": false
        });

        // Make the POST request to the Claude API
        let response = client.post("https://api.anthropic.com/v1/complete")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await?;

        // Extract the response body as a string
        let body = response.text().await?;

        // Deserialize the JSON response
        let json: Value = serde_json::from_str(&body)?;

        // Extract the contents from the JSON response
        if let Some(completion) = json["completion"].as_str() {
            println!("Assistant: {}", completion);

            // Add the assistant's response to the buffer
            buffer.push(json!({
                "role": "assistant",
                "content": completion
            }));

            // Ensure the buffer does not exceed 5 messages
            if buffer.len() > 5 {
                buffer.remove(0); // Remove the oldest message
            }
        }
    }

    Ok(())
}

