// generate.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct CreateImageRequest {
    prompt: String,
    n: usize,
    size: String,
}

#[derive(Deserialize)]
struct CreateImageResponse {
    data: Vec<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    url: String,
}

pub async fn generate_image(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = Client::new();

    let request = CreateImageRequest {
        prompt: prompt.to_string(),
        n: 1,
        size: "1024x1024".to_string(),
    };

    let response = client.post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json::<CreateImageResponse>()
        .await?;

    if let Some(image_data) = response.data.first() {
        Ok(image_data.url.clone())
    } else {
        Err("No image URL returned".into())
    }
}

