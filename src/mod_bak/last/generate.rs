use openai::OpenAIClient;
use openai::openai::OpenAIClientBuilder;
use serde::{Deserialize, Serialize};
use log::info;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct GenerateImageRequest {
    model: String,
    prompt: String,
    size: String,
    quality: String,
    style: String,
}

#[derive(Serialize, Deserialize)]
struct GenerateImageResponseData {
    url: String,
    revised_prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct GenerateImageResponse {
    data: Vec<GenerateImageResponseData>,
}

pub async fn generate_image(prompt: &str, size: &str, quality: &str, style: &str, openai_api_key: &str) -> Result<String, Box<dyn Error>> {
    info!("Entering diffusion model generate image module.");

    let client = OpenAIClientBuilder::new()
        .api_key(openai_api_key.to_string())
        .build()?;

    let request = GenerateImageRequest {
        model: "dall-e-3".to_string(),
        prompt: format!("Generate an image: {}", prompt),
        size: size.to_string(),
        quality: quality.to_string(),
        style: style.to_string(),
    };

    let response: GenerateImageResponse = client
        .post("images/generations")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    if let Some(image_url) = response.data.get(0).map(|data| data.url.clone()) {
        Ok(image_url)
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to generate image")))
    }
}

