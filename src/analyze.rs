use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use log::{info, debug, error};

#[derive(Serialize, Debug)]
struct AnalyzeImageRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum Content {
    Text { r#type: String, text: String },
    ImageUrl { r#type: String, image_url: ImageUrl },
}

#[derive(Serialize, Debug)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize, Debug)]
struct AnalyzeImageResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize, Debug)]
struct MessageResponse {
    content: String,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Deserialize, Debug)]
struct OpenAIErrorResponse {
    error: OpenAIError,
}

#[derive(Deserialize, Debug)]
struct OpenAIError {
    message: String,
}

pub async fn analyze_image(image_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = Client::new();

    let messages = vec![
        Message {
            role: "user".to_string(),
            content: vec![
                Content::Text {
                    r#type: "text".to_string(),
                    text: "Analyze the image in a conversational and friendly manner, explaining your analysis with short details but keep your responses short and concise. Ensure that your responses are properly formatted with bolds and bullet points when they make sense.".to_string(),
                },
                Content::ImageUrl {
                    r#type: "image_url".to_string(),
                    image_url: ImageUrl {
                        url: image_url.to_string(),
                    },
                },
            ],
        },
    ];

    let request = AnalyzeImageRequest {
        model: "gpt-4o".to_string(),
        messages,
    };

    debug!("Sending analyze image request: {:?}", request);

    let response = client.post("https://api.openai.com/v1/chat/completions")
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
        let analyze_response: AnalyzeImageResponse = serde_json::from_str(&response_text)?;
        debug!("Parsed response: {:?}", analyze_response);

        if let Some(usage) = &analyze_response.usage {
            info!("Prompt tokens: {}", usage.prompt_tokens);
            info!("Completion tokens: {}", usage.completion_tokens);
            info!("Total tokens: {}", usage.total_tokens);
        }

        if let Some(choice) = analyze_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No analysis response returned".into())
        }
    } else {
        let error_response: OpenAIErrorResponse = serde_json::from_str(&response_text)?;
        error!("API Error: {:?}", error_response.error);
        Err(error_response.error.message.into())
    }
}

