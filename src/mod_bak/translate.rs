use log::{info, error};
use std::env;
use reqwest::Client;
use whatlang::{detect, Lang};
use tiktoken_rs::CoreBPE;
use serde_json::json;
use uuid::Uuid;
use lazy_static::lazy_static;
use std::collections::HashMap;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

lazy_static! {
    static ref OPENAI_API_KEY: String = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    static ref CLIENT: Client = Client::new();
    static ref TOKENIZER: CoreBPE = {
        let byte_pair_encoder: HashMap<Vec<u8>, usize, BuildHasherDefault<FxHasher>> = HashMap::default(); // replace with actual byte pair encoder map
        let special_tokens: HashMap<String, usize, BuildHasherDefault<FxHasher>> = HashMap::default(); // replace with actual special tokens map
        CoreBPE::new(byte_pair_encoder, special_tokens, "cl100k_base").expect("Failed to initialize tokenizer")
    };
}

pub async fn count_tokens(messages: &[serde_json::Value]) -> Result<usize, String> {
    info!("Counting tokens in messages.");
    let mut num_tokens = 0;
    for message in messages.iter() {
        num_tokens += 3; // tokensPerMessage
        num_tokens += TOKENIZER.encode_ordinary(message["content"].as_str().unwrap_or("")).len();
    }
    let total_tokens = num_tokens + 1; // tokensPerName
    info!("Total tokens counted: {}", total_tokens);
    Ok(total_tokens)
}

pub async fn translate_to_english(text: &str) -> Result<String, String> {
    match detect(text) {
        Some(lang) if lang.lang() == Lang::Eng => {
            info!("Text is already in English.");
            Ok(text.to_string())
        },
        _ => {
            info!("Translating text to English: {}", text);
            let messages = vec![
                json!({"role": "system", "content": "You are a translation assistant."}),
                json!({"role": "user", "content": format!("Translate the following text to English: {}", text)}),
            ];
            match count_tokens(&messages).await {
                Ok(token_count) => {
                    let response = CLIENT.post("https://api.openai.com/v1/chat/completions")
                        .bearer_auth(&*OPENAI_API_KEY)
                        .json(&json!({
                            "model": "gpt-3.5-turbo",
                            "messages": messages,
                            "max_tokens": std::cmp::min(1200, 4096 - token_count)
                        }))
                        .send()
                        .await;
                    match response {
                        Ok(res) => {
                            let response_json: serde_json::Value = res.json().await.unwrap();
                            let translated_text = response_json["choices"][0]["message"]["content"]
                                .as_str()
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            info!("Translation to English generated successfully.");
                            Ok(translated_text)
                        },
                        Err(e) => {
                            error!("Failed to translate text: {:?}", e);
                            Err(format!("Failed to translate text: {:?}", e))
                        }
                    }
                },
                Err(e) => Err(e),
            }
        }
    }
}

pub async fn translate_back_to_user(text: &str, detected_language: &str) -> Result<String, String> {
    info!("Translating text back to {}: {}", detected_language, text);
    let messages = vec![
        json!({"role": "system", "content": "You are a translation assistant."}),
        json!({"role": "user", "content": format!("Translate the following text to {}: {}", detected_language, text)}),
    ];
    match count_tokens(&messages).await {
        Ok(token_count) => {
            let response = CLIENT.post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(&*OPENAI_API_KEY)
                .json(&json!({
                    "model": "gpt-3.5-turbo",
                    "messages": messages,
                    "max_tokens": std::cmp::min(1200, 4096 - token_count)
                }))
                .send()
                .await;
            match response {
                Ok(res) => {
                    let response_json: serde_json::Value = res.json().await.unwrap();
                    let translated_text = response_json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    info!("Translation to detected language generated successfully.");
                    Ok(translated_text)
                },
                Err(e) => {
                    error!("Failed to translate text: {:?}", e);
                    Err(format!("Failed to translate text: {:?}", e))
                }
            }
        },
        Err(e) => Err(e),
    }
}

#[tokio::main]
async fn main() {
    let text_to_translate = "Привет, как дела?";
    match detect(text_to_translate) {
        Some(lang) => info!("Detected language: {}", lang.lang().code()),
        None => error!("Failed to detect language."),
    }

    match translate_to_english(text_to_translate).await {
        Ok(translated_text) => info!("Translated text to English: {}", translated_text),
        Err(e) => error!("Failed to translate text to English: {}", e),
    }

    let text_to_translate_back = "Hello, how are you?";
    match translate_back_to_user(text_to_translate_back, "rus").await {
        Ok(translated_text_back) => info!("Translated text back to rus: {}", translated_text_back),
        Err(e) => error!("Failed to translate text back to rus: {}", e),
    }
}

