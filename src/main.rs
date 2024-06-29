// main.rs 
mod system_prompt;
mod triggers_generate;
mod image_diffusion;
mod image_vision;
mod api_auth;
mod api_routes;
mod input_process; // Add this line

use actix_web::{App, HttpServer, middleware, web};
use std::env;
use log::{info, debug, error};
use log4rs;
use std::fs;
use std::io::{self, Write};
use reqwest::Client;
use dotenv::dotenv;
use serde_json::json;
use std::sync::{Arc, Mutex};
use serde_json::Value;


async fn run_interactive_mode(client: Client, groq_api_key: String, system_prompt: String, shared_messages: Arc<Mutex<Vec<Value>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = shared_messages.lock().unwrap();
    messages.push(json!({
        "role": "system",
        "content": system_prompt
    }));
    debug!("Initial system message set");
    loop {
        print!("\nYou:\n");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim().to_string();
        info!("User input: {}", user_input);

        if user_input.eq_ignore_ascii_case("exit") {
            info!("User requested exit");
            break;
        }

        if let Err(e) = input_process::process_user_input(user_input.clone(), &mut messages, &client, &groq_api_key).await {
            error!("Error processing user input: {}", e);
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs")?;
    // Configure log4rs
    log4rs::init_file("log4rs.yaml", Default::default()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, anyhow::anyhow!(e)))?;

    info!("Starting Fana AI assistant");

    let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    debug!("Loaded GROQ API Key: {}", groq_api_key);

    let system_prompt = system_prompt::SYSTEM_PROMPT.to_string();

    if system_prompt.is_empty() {
        error!("SYSTEM_PROMPT is empty!");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "SYSTEM_PROMPT is empty"));
    }
    debug!("System prompt loaded successfully");

    let client = Client::new();

    // Clone the variables to move them into the thread
    let client_clone = client.clone();
    let groq_api_key_clone = groq_api_key.clone();
    let system_prompt_clone = system_prompt.clone();

    // Initialize the shared message state
    let shared_messages = Arc::new(Mutex::new(Vec::<Value>::new()));

    // Spawn a new thread for the interactive console mode
    let shared_messages_clone = Arc::clone(&shared_messages);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_interactive_mode(client_clone, groq_api_key_clone, system_prompt_clone, shared_messages_clone).await {
                error!("Error in interactive mode: {}", e);
            }
        });
    });

    HttpServer::new(move || {
        let groq_api_key_clone = groq_api_key.clone();
        App::new() 
            .wrap(middleware::Logger::default())
            .wrap(api_auth::ApiKey)
            .configure(move |cfg| {
                api_routes::configure(cfg, groq_api_key_clone.clone())
            })
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::from(shared_messages.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
