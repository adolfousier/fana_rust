// trigger_handler.rs
use crate::image_diffusion::generate_image;
use log::{info, error};
use serde_json::json;
use crate::context_manager::manage_context::ContextManager;
use uuid::Uuid;
use std::net::IpAddr;



pub async fn handle_trigger(user_input: &str, context_manager: &mut ContextManager, ip_addr: IpAddr, session_id: &Uuid) -> Result<String, Box<dyn std::error::Error>> {
    info!("Trigger word detected in user input. Generating image.");

    match generate_image(user_input).await {
        Ok(image_url) => {
            let message = format!("\nFANA:\nI've generated an image based on your request.\nYou can view it here: {}", image_url);
            println!("{}", message);
            info!("Image generated. URL: {}", image_url);
        
            // Add the image information to the conversation
            context_manager.add_message(ip_addr, json!({
                "role": "assistant",
                "content": format!("{}", image_url)
            })).await;
            info!("Added analysis result to context for session {}", session_id);
            Ok(message)
        },
        Err(e) => {
            let message = format!("\nFANA:\nFailed to generate image: {}", e);
            println!("{}", message);
            error!("Image generation failed: {}", e);
            Err(e.into())
        }
    }
}

