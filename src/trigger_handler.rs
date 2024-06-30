// trigger_handler.rs
use crate::image_diffusion::generate_image;
use log::{info, error};
use serde_json::json;
use crate::context_manager::ContextManager;


pub async fn handle_trigger(user_input: &str, context_manager: &mut ContextManager) -> Result<String, Box<dyn std::error::Error>> {
    info!("Trigger word detected in user input. Generating image.");

    match generate_image(user_input).await {
        Ok(image_url) => {
            let message = format!("\nFANA:\nI've generated an image based on your request.\nYou can view it here: {}", image_url);
            println!("{}", message);
            //println!("\nFANA:\nI've generated an image based on your request.");
            //println!("You can view it here: {}", image_url);
            info!("Image generated. URL: {}", image_url);
        
            // Add the image information to the conversation
            context_manager.add_message(json!({
                "role": "assistant",
                "content": format!("{}", image_url)
            })).await;

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
