use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;
use log::{info, error};
use tiktoken_rs::cl100k_base;
use tiktoken_rs::CoreBPE;

#[derive(Serialize, Deserialize)]
struct MessageContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    image_url: Option<ImageURL>,
}

#[derive(Serialize, Deserialize)]
struct ImageURL {
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<MessageContent>,
}

#[derive(Serialize, Deserialize)]
struct Configuration {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Serialize, Deserialize)]
struct ChoiceMessage {
    content: String,
}

async fn count_tokens(messages: &[Message]) -> usize {
    let bpe = cl100k_base().unwrap();
    let mut total_tokens = 0;

    for message in messages {
        // Count tokens for the role
        total_tokens += bpe.encode_ordinary(&message.role).len();

        // Count tokens for each content item
        for content in &message.content {
            match content.content_type.as_str() {
                "text" => {
                    if let Some(text) = &content.text {
                        total_tokens += bpe.encode_ordinary(text).len();
                    }
                }
                "image_url" => {
                    // Add a fixed number of tokens for image URL (you may adjust this)
                    total_tokens += 100;
                }
                _ => {}
            }
        }
    }

    total_tokens
}

async fn count_response_tokens(response: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_ordinary(response).len()
}

pub async fn analyze_image(image_url: String, message: String) -> Result<String, String> {
    info!("Entering analyze image computer vision module.");
    let api_url = "https://api.openai.com/v1/chat/completions";
    let configuration = Configuration {
        model: "gpt-4".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: vec![
                    MessageContent {
                        content_type: "text".to_string(),
                        text: Some("Analyse the image in a conversational and friendly manner, explaining your analysis with short details but keep your responses short and concise. Ensure that your responses are properly formatted with bolds and bullet points when make sense.".to_string()),
                        image_url: None,
                    },
                    MessageContent {
                        content_type: "image_url".to_string(),
                        text: None,
                        image_url: Some(ImageURL { url: image_url.clone() }),
                    }
                ],
            }
        ],
        max_tokens: 1000,
    };
    
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new();
    let response = client.post(api_url)
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .header("Content-Type", "application/json")
        .json(&configuration)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let description: ApiResponse = resp.json().await.unwrap();
                let analysis_result = &description.choices[0].message.content;
                info!("Analyze image module processed successfully. Analysis result: {}", analysis_result);
                
                // Count tokens used for input
                let tokens_used_input = count_tokens(&configuration.messages).await;
                info!("Tokens used for input in analyze image: {}", tokens_used_input);
                
                // Count tokens used for output
                let tokens_used_output = count_response_tokens(&analysis_result).await;
                info!("Tokens used for output in analyze image: {}", tokens_used_output);
                
                let total_tokens_used = tokens_used_input + tokens_used_output;
                info!("Total tokens used in analyze image: {}", total_tokens_used);
                
                Ok(analysis_result.clone())
            } else {
                let error_message = format!("Error: {}", resp.status());
                error!("{}", error_message);
                Err(error_message)
            }
        }
        Err(e) => {
            let error_message = format!("Error analyzing image: {}", e);
            error!("{}", error_message);
            Err(error_message)
        }
    }
}

