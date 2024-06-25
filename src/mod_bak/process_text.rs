use crate::content_fetcher::find_most_similar_content;
use crate::system_prompt::get_system_prompt;
use crate::embedding::generate_embedding;
use crate::sb_client::query_supabase;
use crate::sb_chat_history::{retrieve_chat_history, store_chat_history};
use crate::chat_completion::generate_chat_response;
use crate::translate::{translate_to_english, translate_back_to_user};
use crate::triggers_check::check_for_trigger_words;
use crate::process_image::process_image_interaction
use log::{info, error};
use whatlang::detect;
use serde_json::{Value, json};
use tiktoken_rs::CoreBPE;
use std::env;
use lazy_static::lazy_static;
use openai_rust::chat::Message;

lazy_static! {
    static ref OPENAI_API_KEY: String = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    static ref TOKENIZER: CoreBPE = CoreBPE::new("cl100k_base").expect("Failed to initialize tokenizer");
}

pub async fn count_tokens(messages: Vec<Value>) -> usize {
    info!("Counting tokens in messages.");
    let mut num_tokens = 0;
    for message in messages.iter() {
        num_tokens += 3; // tokensPerMessage
        num_tokens += TOKENIZER.encode_ordinary(message["content"].as_str().unwrap_or("")).len();
    }
    info!("Total tokens counted: {}", num_tokens + 1);
    num_tokens + 1 // tokensPerName
}

pub async fn retrieve_system_prompt() -> Result<String, String> {
    info!("Retrieving system prompt.");
    match get_system_prompt().await {
        Ok(system_prompt) => {
            info!("System prompt retrieved successfully.");
            Ok(system_prompt)
        },
        Err(e) => {
            error!("Failed to retrieve system prompt: {}", e);
            Err(format!("Failed to retrieve system prompt: {}", e))
        }
    }
}

pub async fn generate_user_input_embedding(user_input: &str) -> Result<Vec<f32>, String> {
    info!("Generating embedding for user input.");
    match generate_embedding(user_input).await {
        Ok(embedding) => {
            info!("Generated embedding for user input successfully.");
            Ok(embedding)
        },
        Err(e) => {
            error!("Failed to generate embedding for user input: {}", e);
            Err(format!("Failed to generate embedding for user input: {}", e))
        }
    }
}

pub async fn get_conversation_history(
    user_input_embedding: Vec<f32>, 
    user_id: &str, 
    session_id: &str, 
    user_input: &str
) -> Vec<Value> {
    info!("Retrieving conversation history using embeddings.");
    match retrieve_chat_history(user_id, session_id, Some(user_input_embedding), Some(user_input.to_string())).await {
        Ok(conversation_history) => {
            info!("Retrieved conversation history from Supabase successfully.");
            conversation_history.unwrap_or_else(Vec::new)
        },
        Err(e) => {
            error!("Failed to retrieve conversation history: {}", e);
            Vec::new()
        }
    }
}

pub async fn query_supabase_data() -> Vec<Value> {
    info!("Querying Supabase for data.");
    match query_supabase().await {
        Ok(data) => {
            info!("Queried Supabase successfully.");
            data
        },
        Err(e) => {
            error!("Failed to query Supabase: {}", e);
            Vec::new()
        }
    }
}

pub async fn find_similar_content(
    updated_user_input_embedding: Vec<f32>, 
    supabase_data: Vec<Value>
) -> Option<String> {
    info!("Finding most similar content in Supabase.");
    match find_most_similar_content(updated_user_input_embedding, supabase_data).await {
        Ok(content) => {
            info!("Found most similar content: {}", content);
            Some(content)
        },
        Err(e) => {
            error!("Failed to find most similar content: {}", e);
            None
        }
    }
}

pub async fn store_updated_chat_history(
    conversation_history: Vec<Value>, 
    user_id: &str, 
    session_id: &str
) {
    info!("Storing updated chat history in Supabase.");
    let chat_contents: Vec<String> = conversation_history.iter().map(|msg| msg["content"].as_str().unwrap_or("").to_string()).collect();
    match generate_embedding(&chat_contents).await {
        Ok(updated_chat_history_embeddings) => {
            if let Err(e) = store_chat_history(&conversation_history, &updated_chat_history_embeddings, user_id, session_id).await {
                error!("Failed to store chat history: {}", e);
            } else {
                info!("Stored chat history in Supabase successfully.");
            }
        },
        Err(e) => error!("Failed to generate embeddings for chat history: {}", e),
    }
}

pub async fn handle_text_interaction(
    user_input: &str, 
    chat_history: Vec<Value>, 
    user_id: &str, 
    session_id: &str
) -> (String, Vec<Value>) {
    info!("Received user input, handling text interaction.");

    let detected_language = match detect(user_input) {
        Some(lang) => {
            let lang_str = lang.to_string();
            info!("Detected language: {}", lang_str);
            lang_str
        },
        None => {
            error!("Failed to detect language.");
            "en".to_string() // Default to English if detection fails
        }
    };

    let translated_input = if detected_language != "en" {
        match translate_to_english(user_input).await {
            Ok(translated) => translated,
            Err(_) => return ("Error translating user input to English".to_string(), vec![]),
        }
    } else {
        user_input.to_string()
    };

    let (triggers_found, found_triggers) = check_for_trigger_words(&translated_input).await;
    if triggers_found {
        info!("Trigger words detected, proceeding with image generation.");
        match process_image_interaction(&translated_input, &chat_history, user_id, session_id).await {
            Ok(image_response) => {
                let response = if detected_language != "en" {
                    match translate_back_to_user(&image_response, &detected_language).await {
                        Ok(translated) => translated,
                        Err(_) => return ("Error translating response back to the detected language".to_string(), vec![]),
                    }
                } else {
                    image_response
                };

                let new_conversation_history = vec![
                    json!({ "role": "user", "content": user_input }),
                    json!({ "role": "assistant", "content": response })
                ];

                store_updated_chat_history(new_conversation_history.clone(), user_id, session_id).await;

                return (response, [chat_history, new_conversation_history].concat());
            },
            Err(e) => {
                error!("Error during image generation: {}", e);
                return ("Error generating image".to_string(), vec![]);
            }
        }
    }

    let system_prompt = match retrieve_system_prompt().await {
        Ok(prompt) => prompt,
        Err(_) => return ("Error retrieving system prompt".to_string(), vec![]),
    };

    let user_input_embedding = match generate_user_input_embedding(&translated_input).await {
        Ok(embedding) => embedding,
        Err(_) => return ("Error generating embedding for user input".to_string(), vec![]),
    };

    let retrieved_history = get_conversation_history(user_input_embedding.clone(), user_id, session_id, &translated_input).await;
    let context_text = retrieved_history.iter().map(|msg| msg["content"].as_str().unwrap_or("")).collect::<Vec<&str>>().join(" ");
    let updated_user_input = format!("{} {}", context_text, translated_input);

    let updated_user_input_embedding = match generate_user_input_embedding(&updated_user_input).await {
        Ok(embedding) => embedding,
        Err(_) => return ("Error generating embedding for updated user input".to_string(), vec![]),
    };

    let supabase_data = query_supabase_data().await;
    if supabase_data.is_empty() {
        return ("Error querying Supabase".to_string(), vec![]);
    }

    let most_similar_content = find_similar_content(updated_user_input_embedding, supabase_data).await;
    let messages = if let Some(content) = most_similar_content {
        vec![
            json!({ "role": "system", "content": system_prompt }),
            json!({ "role": "user", "content": updated_user_input }),
            json!({ "role": "assistant", "content": format!(
                "Based on the user input, and most similar content found, rewrite it gracefully. Always provide a friendly, engaging and concise response.
                - Remember: You're the AImagine AI assistant; a fun and friendly advanced AI assistant part of the by AImagine team!
                - Ensure you respond as part of the team always with \"we\", \"us\" to create a sense of teamwork.
                - When users send messages in any specific language, ensure to reply in the same language.
                - DO NOT send large responses; keep it under 3-4 sentences max.
                - Remember, keep your responses short; keeping our guidance clear and concise.") }),
        ]
    } else {
        vec![
            json!({ "role": "system", "content": system_prompt }),
            json!({ "role": "user", "content": updated_user_input }),
        ]
    };

    let tokens_used = count_tokens(messages.clone()).await;
    info!("Tokens: {}", tokens_used);

    let generated_response = match generate_chat_response(messages).await {
        Ok(response) => response,
        Err(_) => return ("Error generating chat completion".to_string(), vec![]),
    };

    let translated_response = if detected_language != "en" {
        match translate_back_to_user(&generated_response, &detected_language).await {
            Ok(translated) => translated,
            Err(_) => return ("Error translating response back to the detected language".to_string(), vec![]),
        }
    } else {
        generated_response
    };

    let new_conversation_history = vec![
        json!({ "role": "user", "content": user_input }),
        json!({ "role": "assistant", "content": translated_response })
    ];

    store_updated_chat_history(new_conversation_history.clone(), user_id, session_id).await;

    let complete_history = [retrieved_history, new_conversation_history].concat();

    (translated_response, complete_history)
}

