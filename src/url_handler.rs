// url_handler.rs
use crate::image_vision::analyze_image;
use log::{info, error};
use regex::Regex;
use serde_json::json;
use crate::context_manager::ContextManager;

pub async fn handle_url(url: &str, context_manager: &mut ContextManager) -> Result<String, Box<dyn std::error::Error>> {
    info!("URL detected in user input: {}", url);

    match analyze_image(url).await {
        Ok(analysis) => {
            println!("\nFANA:\nImage analysis: {}", analysis);
            info!("Image analysis: {}", analysis);
                
            // Add the analysis result to the conversation
            context_manager.add_message(json!({
                "role": "assistant",
                "content": analysis
            })).await;
            
            Ok(analysis)
        },
        Err(e) => {
            println!("\nFANA:\n{}", e);
            error!("Image analysis failed: {}", e);
            Err(e.into())
        }
    }
}


pub fn contains_url(text: &str) -> Option<&str> {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    url_regex.find(text).map(|m| m.as_str())
}
