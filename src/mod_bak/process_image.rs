use crate::modules::triggers_check::check_for_trigger_words;
use crate::modules::embedding::generate_embedding;
use crate::modules::generate::generate_image;
use crate::modules::analyze::analyze_image;
use crate::modules::sb_chat_history::{store_chat_history, retrieve_chat_history};
use actix_multipart::Multipart;
use log::{info, error};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;


pub async fn retrieve_conversation_history(
    user_input_embedding: Vec<f32>, 
    user_id: &str, 
    session_id: &str, 
    user_input: &str
) -> Vec<Value> {
    info!("Retrieving conversation history using embeddings.");
    match retrieve_chat_history(user_id, session_id, Some(user_input_embedding), Some(user_input.to_string())).await {
        Ok(conversation_history) => {
            info!("Retrieved conversation history successfully.");
            conversation_history.unwrap_or_else(Vec::new)
        },
        Err(e) => {
            error!("Failed to retrieve conversation history: {}", e);
            Vec::new()
        }
    }
}

pub async fn update_and_store_chat_history(
    chat_history: Vec<Value>, 
    user_id: &str, 
    session_id: &str
) {
    info!("Storing updated chat history in Supabase.");
    let chat_contents: Vec<String> = chat_history.iter().map(|msg| msg["content"].as_str().unwrap_or("").to_string()).collect();
    match generate_embedding(&chat_contents).await {
        Ok(updated_chat_history_embeddings) => {
            if let Err(e) = store_chat_history(&chat_history, &updated_chat_history_embeddings, user_id, session_id).await {
                error!("Failed to store chat history: {}", e);
            } else {
                info!("Stored chat history in Supabase successfully.");
            }
        },
        Err(e) => error!("Failed to generate embeddings for chat history: {}", e),
    }
}

pub async fn handle_message_with_triggers(
    message: &str,
    analysis_result: &str,
    mut chat_history: Vec<Value>,
    user_input: &str,
    retrieved_history: Vec<Value>,
    user_id: &str,
    session_id: &str,
) -> (String, Vec<Value>) {
    let trigger_check_result = check_for_trigger_words(message, user_input).await;
    let triggers_found = trigger_check_result.0;

    info!(
        "Input processed: '{}', Triggers found: {:?}, Is question: {}",
        message, trigger_check_result.1, trigger_check_result.0
    );
    if triggers_found {
        info!(
            "Trigger words identified along with URL, proceeding to generate image based on analysis."
        );
        let image_prompt = format!(
            "{} {} {}",
            analysis_result,
            message,
            retrieved_history.iter().map(|msg| msg["content"].as_str().unwrap_or("")).collect::<Vec<&str>>().join(" ")
        );
        match generate_image(&image_prompt).await {
            Ok(image_url) => {
                info!("Image generated successfully");
                let response = format!("{} \n {}", image_url, analysis_result);
                chat_history.push(json!({ "role": "assistant", "content": response }));
                return (response, chat_history);
            },
            Err(e) => {
                error!("Failed to generate image: {}", e);
                chat_history.push(json!({ "role": "assistant", "content": "Sorry, I couldn't generate the image, try a different prompt please." }));
                return ("Sorry, I couldn't generate the image.".to_string(), chat_history);
            }
        }
    } else {
        info!("No trigger words found in the message. Proceeding with the regular analysis result response.");
        return return_analysis_result(analysis_result, chat_history, user_input).await;
    }
}

pub async fn handle_no_url(
    user_input: &str,
    mut chat_history: Vec<Value>,
    user_id: &str,
    session_id: &str,
) -> (String, Vec<Value>) {
    info!("No URLs found in the user input, checking for trigger words");

    chat_history.push(json!({ "role": "user", "content": user_input }));

    let user_input_embedding = generate_embedding(&[user_input]).await.unwrap_or_else(|_| vec![]);
    let retrieved_history = retrieve_conversation_history(user_input_embedding.clone(), user_id, session_id, user_input).await;
    let combined_input_with_history = format!("{} {}", user_input, retrieved_history.iter().map(|msg| msg["content"].as_str().unwrap_or("")).collect::<Vec<&str>>().join(" "));

    info!("Combined input for trigger check: {}", combined_input_with_history);

    let trigger_check_result = check_for_trigger_words(&combined_input_with_history).await;
    let triggers_found = trigger_check_result.0;

    info!(
        "Input processed: {}, Triggers found: {:?}, Is question: {}",
        user_input, trigger_check_result.1, trigger_check_result.0
    );
    if triggers_found {
        info!("Trigger words found in user input without URLs.");
        let image_prompt = combined_input_with_history;
        info!("Entering generate image module with trigger words. Image prompt: {}", image_prompt);
        match generate_image(&image_prompt).await {
            Ok(image_url) => {
                let response = format!("{}", image_url);
                chat_history.push(json!({ "role": "assistant", "content": response }));
                info!("Generate image module processed successfully with only trigger words.");
                return (response, chat_history);
            },
            Err(e) => {
                error!("Failed to generate image: {}", e);
                chat_history.push(json!({ "role": "assistant", "content": "Sorry, I couldn't generate the image, try a different prompt please." }));
                return ("Sorry, I couldn't generate the image.".to_string(), chat_history);
            }
        }
    }
    info!("Unable to process the image. No trigger words found with no URL in the user-input.");
    update_and_store_chat_history(chat_history.clone(), user_id, session_id).await;
    ("Sorry, I couldn't understand your request. Please try again.".to_string(), chat_history)
}

pub async fn analyze_and_generate(
    user_input: &str,
    mut chat_history: Vec<Value>,
    url: &str,
    message: &str,
    user_id: &str,
    session_id: &str,
) -> (String, Vec<Value>) {
    info!("Sending a request to analyze image module.");

    let user_input_embedding = generate_embedding(&[user_input]).await.unwrap_or_else(|_| vec![]);
    let retrieved_history = retrieve_conversation_history(user_input_embedding.clone(), user_id, session_id, user_input).await;

    let combined_input = format!("{} {}", user_input, retrieved_history.iter().map(|msg| msg["content"].as_str().unwrap_or("")).collect::<Vec<&str>>().join(" "));

    info!("Combined input for analysis: {}", combined_input);

    match analyze_image(url, &combined_input).await {
        Ok(analysis_result) => {
            info!("Analyze image process successful.");
            if !message.is_empty() {
                let (response, updated_chat_history) = handle_message_with_triggers(
                    message,
                    &analysis_result,
                    chat_history.clone(),
                    user_input,
                    retrieved_history,
                    user_id,
                    session_id,
                ).await;
                if !response.is_empty() {
                    return (response, updated_chat_history);
                }
            }
            return return_analysis_result(&analysis_result, chat_history.clone(), user_input).await;
        },
        Err(e) => {
            error!("Failed to analyze image: {}", e);
            return (format!("Sorry, I couldn't analyze the image at {}.", url), chat_history);
        }
    }
}

pub async fn return_analysis_result(
    analysis_result: &str,
    mut chat_history: Vec<Value>,
    user_input: &str,
) -> (String, Vec<Value>) {
    info!("Returning analysis result");
    let response = format!("{}", analysis_result);
    chat_history.push(json!({ "role": "assistant", "content": response }));
    (response, chat_history)
}

pub async fn process_image_interaction(
    user_input: &str,
    mut chat_history: Vec<Value>,
    user_id: &str,
    session_id: &str,
    image: Option<Multipart>,
) -> (String, Vec<Value>) {
    info!("Entering process image interaction module");
    let url_match = Regex::new(r"(https?://\S+)").unwrap().find(user_input);
    if let Some(url) = url_match {
        let url_str = url.as_str();
        let message = user_input.replace(url_str, "").trim().to_string();
        chat_history.push(json!({ "role": "user", "content": user_input }));
        analyze_and_generate(user_input, chat_history.clone(), url_str, &message, user_id, session_id).await
    } else {
        handle_no_url(user_input, chat_history.clone(), user_id, session_id).await
    }
}

