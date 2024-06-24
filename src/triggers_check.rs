use crate::modules::logging_setup::setup_logging;
use crate::modules::triggers_generate::trigger_words;
use crate::modules::triggers_regenerate::get_regeneration_patterns;
use log::{info, error};
use regex::Regex;
use std::collections::HashSet;
use std::env;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

lazy_static! {
    static ref OPENAI_API_KEY: String = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    static ref REGENERATION_PATTERNS: Vec<String> = {
        let patterns = get_regeneration_patterns();
        patterns.iter().map(|word| regex::escape(word)).collect()
    };
    static ref REGENERATION_REGEX: Regex = Regex::new(&REGENERATION_PATTERNS.join("|")).unwrap();
    static ref REGENERATION_REGEX_PATTERNS: Vec<Regex> = {
        let patterns = get_regeneration_patterns();
        patterns.iter().map(|word| {
            Regex::new(&format!(
                r"(another one|one more|add a|change the|last image|replace the|try other)\s+(.*?\s+)?({})",
                regex::escape(word)
            )).unwrap()
        }).collect()
    };
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

pub fn extract_phrases(text: &str) -> HashSet<String> {
    let phrases: HashSet<String> = text
        .to_lowercase()
        .split(|c: char| ['.', ',', '!', '?', ';', ':'].contains(&c))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    phrases
}

pub fn contains_exact_trigger_pattern(text: &str, patterns: &[String]) {
    let found_patterns: Vec<&String> = patterns
        .iter()
        .filter(|&pattern| text.to_lowercase().contains(&pattern.to_lowercase()))
        .collect();

    if !found_patterns.is_empty() {
        info!("Trigger patterns found: {:?}", found_patterns);
    }
}

pub fn regeneration_combine(user_input: &str, last_entry: Option<&serde_json::Value>) -> String {
    if let Some(entry) = last_entry {
        if let Some(chat_history) = entry.get("chat_history") {
            if let Some(data) = chat_history.get("data") {
                if let Some(last_item) = data.as_array().and_then(|arr| arr.last()) {
                    if let Some(content) = last_item.get("content") {
                        let combined_input = format!(
                            "To generate the image, take into consideration the following user input: \"{}\" and additionally add the following description to the image: \"{}\".",
                            user_input, content.as_str().unwrap_or("")
                        );
                        info!("Combined input for LLM: {}", combined_input);
                        return combined_input;
                    }
                }
            }
        }
        error!("Invalid or empty chat history data.");
    } else {
        error!("Invalid last entry format or data missing.");
    }
    "No similar content found".to_string()
}

pub async fn check_for_trigger_words(
    user_input: Option<&str>,
    combined_input: Option<&str>,
) -> (bool, Vec<String>) {
    let user_input_lower = if let Some(input) = combined_input {
        input.to_lowercase()
    } else if let Some(input) = user_input {
        input.to_lowercase()
    } else {
        return (false, vec![]);
    };

    let found_triggers: Vec<String> = trigger_words()
        .into_iter()
        .filter(|trigger| user_input_lower.contains(trigger))
        .collect();

    let is_question = REGENERATION_REGEX_PATTERNS
        .iter()
        .any(|pattern| pattern.is_match(&user_input_lower));

    info!(
        "Input processed: '{}', Triggers found: {:?}, Is question: {}",
        user_input_lower, found_triggers, is_question
    );

    if !found_triggers.is_empty() || is_question {
        info!("Trigger words or question pattern detected for action; proceeding with image generation.");
        (true, found_triggers)
    } else {
        (false, vec![])
    }
}


pub fn initialize_logging() {
    setup_logging().expect("Failed to initialize logging");
}

