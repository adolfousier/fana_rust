use serde::{Deserialize, Serialize};
use tiktoken_rs::cl100k_base;
use tiktoken_rs::CoreBPE;
use lazy_static::lazy_static;
use std::env;
use log::{info, error};
use openai_rust::{Client, chat::{ChatArguments, Message}};

lazy_static! {
    static ref OPENAI_API_KEY: String = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    static ref CLIENT: Client = Client::new(&OPENAI_API_KEY);
    static ref TOKENIZER: CoreBPE = cl100k_base().expect("Failed to load tokenizer");
}

#[derive(Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Serialize, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

pub async fn count_tokens(messages: Vec<Message>) -> usize {
    info!("Counting tokens in messages.");
    let mut num_tokens = 0;
    for message in messages.iter() {
        num_tokens += 3;  // tokensPerMessage for the system
        num_tokens += TOKENIZER.encode(&message.content).len();  // Accurate token count using tokenizer
    }
    let total_tokens = num_tokens + 1;  // tokensPerName
    info!("Total tokens counted: {}", total_tokens);
    total_tokens
}

pub async fn generate_chat_response(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Sending request to LLM to generate chat completion.");

    let token_count = count_tokens(messages.clone()).await;
    info!("Token count: {}", token_count);

    let args = ChatArguments::new("gpt-3.5-turbo", messages)
        .max_tokens(1200.min(4096 - token_count)); // Ensure max_tokens does not exceed limit

    match CLIENT.create_chat(args).await {
        Ok(response) => {
            if let Some(choice) = response.choices.get(0) {
                let generated_response = choice.content.trim().to_string();
                info!("Chat response generated successfully.");
                Ok(generated_response)
            } else {
                error!("Failed to generate chat completion: No choices returned");
                Err("Failed to generate chat completion: No choices returned".into())
            }
        },
        Err(err) => {
            let error_message = format!("Failed to generate chat completion: {}", err);
            error!("{}", error_message);
            Err(Box::new(err))
        }
    }
}

