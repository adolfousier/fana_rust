use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct CreateImageRequest {
    prompt: String,
    n: usize,
    size: String,
    model: String, // Add model field to speciftype: "text"y DALL-E 3
}

#[derive(Deserialize)]
struct CreateImageResponse {
    data: Vec<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    url: String,
}

fn generation_prompt(user_input: &str) -> String {
    format!("Create a visually stunning and detailed image based on the following prompt: {}", user_input)
}

pub async fn generate_image(user_input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = Client::new();

    let prompt = generation_prompt(user_input);

    let request = CreateImageRequest {
        prompt,
        n: 1,
        size: "1024x1024".to_string(),
        model: "dall-e-3".to_string(), // Specify DALL-E 3 model
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

