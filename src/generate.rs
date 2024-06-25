use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use log::{info, debug, error};
use serde_json::Value;

#[derive(Serialize, Debug)]
struct CreateImageRequest {
    prompt: String,
    n: usize,
    size: String,
}

#[derive(Deserialize, Debug)]
struct CreateImageResponse {
    data: Vec<ImageData>,
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
struct ImageData {
    url: String,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

fn generation_prompt(user_input: &str) -> String {
    format!("Create a visually stunning and high quality image based on the following prompt: {}", user_input)
}

pub async fn generate_image(user_input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = Client::new();

    let prompt = generation_prompt(user_input);

    let request = CreateImageRequest {
        prompt,
        n: 1,
        size: "1024x1024".to_string(),
    };

    debug!("Sending generate image request: {:?}", request);

    let response = client.post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    debug!("Received response: {:?}", response);

    let status = response.status();
    let response_text = response.text().await?;
    debug!("Response text: {}", response_text);

    if status.is_success() {
        let generate_response: CreateImageResponse = serde_json::from_str(&response_text)?;
        debug!("Parsed response: {:?}", generate_response);

        if let Some(usage) = &generate_response.usage {
            info!("Prompt tokens: {}", usage.prompt_tokens);
            info!("Completion tokens: {}", usage.completion_tokens);
            info!("Total tokens: {}", usage.total_tokens);
        }

        if let Some(image_data) = generate_response.data.first() {
            Ok(image_data.url.clone())
        } else {
            Err("No image URL returned".into())
        }
    } else {
        let error_response: Value = serde_json::from_str(&response_text)?;
        error!("API Error: {:?}", error_response);
        Err("Failed to generate image".into())
    }
}


