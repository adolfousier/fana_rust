use crate::modules::embedding::cosine_similarity;
use log::{info, error};
use serde_json::Value;
use crate::triggers_check::{regeneration_combine, contains_exact_trigger_pattern};
use crate::triggers_regenerate::get_regeneration_patterns;
use regex::Regex;
use std::error::Error;
use std::collections::HashSet;
use futures::future;
use ndarray::Array1;



fn normalize_embedding(embedding: Vec<f32>) -> Vec<f32> {
    let norm = embedding.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    embedding.into_iter().map(|x| x / norm).collect()
}

fn extract_and_normalize_embeddings(supabase_data: &[Value]) -> (Vec<Vec<f32>>, Vec<String>) {
    let mut entry_embeddings = Vec::new();
    let mut entry_ids = Vec::new();

    for entry in supabase_data {
        if let Some(entry_embedding_str) = entry.get("embedding").and_then(Value::as_str) {
            if entry_embedding_str == "[]" {
                error!("Embedding string is empty.");
                continue;
            }
            if let Ok(entry_embedding) = serde_json::from_str::<Vec<f32>>(entry_embedding_str) {
                if entry_embedding.is_empty() {
                    error!("Invalid or empty embedding array for entry");
                    continue;
                }
                let normalized_embedding = normalize_embedding(entry_embedding);
                entry_embeddings.push(normalized_embedding);
                entry_ids.push(entry.get("id").and_then(Value::as_str).unwrap_or("unknown").to_string());
            } else {
                error!("Error parsing embedding string.");
            }
        } else {
            error!("No embedding found for entry.");
        }
    }

    (entry_embeddings, entry_ids)
}

async fn compute_similarities(user_embedding: Vec<f32>, entry_embeddings: Vec<Vec<f32>>) -> Vec<f32> {
    let tasks: Vec<_> = entry_embeddings.into_iter()
        .map(|entry_embedding| cosine_similarity(user_embedding.clone(), entry_embedding))
        .collect();

    future::join_all(tasks).await
}

fn boost_similarity_for_key_phrases(user_input: &str, similarities: &mut [f32], supabase_data: &[Value]) {
    let key_phrases = ["my name is", "what is my name"];
    let user_input_lower = user_input.to_lowercase();

    for (i, entry) in supabase_data.iter().enumerate() {
        if let Some(chat_history) = entry.get("chat_history").and_then(|ch| ch.get("data")).and_then(Value::as_array) {
            let entry_text = chat_history.iter()
                .filter_map(|msg| msg.get("content").and_then(Value::as_str))
                .map(|content| content.to_lowercase())
                .collect::<Vec<_>>()
                .join(" ");
            
            for key_phrase in &key_phrases {
                if user_input_lower.contains(key_phrase) && entry_text.contains(key_phrase) {
                    info!("Boosting similarity for key phrase: {} in entry ID: {}", key_phrase, entry.get("id").unwrap_or(&Value::Null));
                    similarities[i] = 1.0;
                }
            }
        }
    }
}

pub async fn find_most_similar_content(user_embedding: Vec<f32>, supabase_data: Vec<Value>) -> Result<String, Box<dyn Error>> {
    info!("Entering find most similar content module.");
    
    let normalized_user_embedding = normalize_embedding(user_embedding);
    info!("Normalize user embedding processed successfully");

    let (entry_embeddings, entry_ids) = extract_and_normalize_embeddings(&supabase_data);
    info!("Extracted and normalized entry embeddings: {:?}", entry_embeddings);
    info!("Entry IDs: {:?}", entry_ids);

    info!("Computing similarities...");
    let similarities = compute_similarities(normalized_user_embedding, entry_embeddings).await;
    info!("Similarities: {:?}", similarities);

    if similarities.is_empty() {
        info!("No valid similarities found.");
        return Ok("No similar content found due to invalid data".to_string());
    }

    let max_similarity = similarities.iter().cloned().fold(0./0., f32::max);
    let similarity_threshold = 0.55;

    if max_similarity < similarity_threshold {
        info!("No similar content found above the similarity threshold.");
        return Ok("No similar content found".to_string());
    }

    let max_index = similarities.iter().position(|&x| x == max_similarity).unwrap();
    info!("Max similarity index: {}, Entry ID: {}, Max similarity: {}", max_index, entry_ids[max_index], max_similarity);

    let most_similar_content = supabase_data[max_index].get("content").and_then(Value::as_str).unwrap_or("No content found").to_string();
    
    for (idx, similarity) in similarities.iter().enumerate() {
        info!("Entry ID: {}, Similarity: {}", entry_ids[idx], similarity);
    }

    info!("Found most similar content with similarity: {}", max_similarity);
    Ok(most_similar_content)
}

pub async fn find_most_similar_chat_history(user_embedding: Vec<f32>, supabase_data: Vec<Value>, user_input: &str) -> Result<String, Box<dyn Error>> {
    info!("Entering find most similar chat history module.");

    let normalized_user_embedding = normalize_embedding(user_embedding);
    info!("Normalize user embedding processed successfully.");
    
    let (entry_embeddings, entry_ids) = extract_and_normalize_embeddings(&supabase_data);
    info!("Extracted and normalized entry embeddings: {:?}", entry_embeddings);
    info!("Entry IDs: {:?}", entry_ids);

    info!("Computing similarities...");
    let mut similarities = compute_similarities(normalized_user_embedding, entry_embeddings).await;
    info!("Similarities before boosting: {:?}", similarities);
    boost_similarity_for_key_phrases(user_input, &mut similarities, &supabase_data);
    info!("Similarities after boosting: {:?}", similarities);

    let similarity_threshold = 0.70;
    let exact_triggers = contains_exact_trigger_pattern(user_input, &get_regeneration_patterns());

    if exact_triggers {
        info!("Regeneration trigger found. Retrieving the last image description.");
        if let Some(last_entry) = supabase_data.last() {
            let combined_input = regeneration_combine(user_input, Some(last_entry));
            info!("Returning combined input for LLM: {}", combined_input);
            return Ok(combined_input);
        }
    }

    let max_similarity = similarities.iter().cloned().fold(0./0., f32::max);

    if max_similarity < similarity_threshold {
        info!("No similar chat history found above the similarity threshold.");
        return Ok("No similar content found".to_string());
    }

    let max_index = similarities.iter().position(|&x| x == max_similarity).unwrap();
    info!("Max similarity index: {}, Entry ID: {}, Max similarity score: {}", max_index, entry_ids[max_index], max_similarity);

    let mut most_similar_chat_history = String::new();
    if let Some(data) = supabase_data[max_index].get("chat_history").and_then(|ch| ch.get("data")).and_then(Value::as_array) {
        for entry in data {
            if let (Some(role), Some(content)) = (entry.get("role").and_then(Value::as_str), entry.get("content").and_then(Value::as_str)) {
                most_similar_chat_history.push_str(&format!("{}: {}\n", role, content));
            }
        }
    }

    info!("Most similar chat history: {}", most_similar_chat_history);
    let combined_input = format!("{} {}", process_text(user_input), process_text(&most_similar_chat_history));
    info!("Combined input for LLM: {}", combined_input);
    Ok(combined_input)
}

fn process_text(text: &str) -> String {
    Regex::new(r"http\S+").unwrap().replace_all(text, "").trim().to_string()
}

fn combine_input(user_input: &str, last_entry: &Value) -> String {
    let parts: Vec<&str> = user_input.split("? ").collect();
    if parts.len() > 1 {
        let query = parts[1].trim();
        if let Some(chat_history) = last_entry.get("chat_history").and_then(|ch| ch.get("data")) {
            format!("{} {}", chat_history, query)
        } else {
            user_input.to_string()
        }
    } else {
        user_input.to_string()
    }
}

