use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Error, get, post, middleware::Logger, HttpRequest};
use actix_cors::Cors;
use serde_json::json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, error};
use std::env;
use actix_web::http::header::{COOKIE, SET_COOKIE};
use reqwest::Client;
use crate::modules::api_auth::get_api_key;
use crate::modules::logging_setup::setup_logging;
use crate::DESCRIPTION;

#[derive(Serialize, Deserialize)]
struct TextRequest {
    text: Option<String>,
    chat_history: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct LLMResponse {
    response: Option<String>,
    chat_history: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct ImageUploadResponse {
    image_url: String,
}

#[derive(Serialize, Deserialize)]
struct ImageRequest {
    image_url: String,
}

#[derive(Serialize, Deserialize)]
struct ApiKey {
    key: String,
}

async fn read_main(_api_key: ApiKey) -> impl Responder {
    info!("Interacted with AImagine LLM POST");
    HttpResponse::Ok().json(json!({"msg": "Hello from AImagine API V1"}))
}

async fn interact_with_llm(
    req: HttpRequest,
    text: Option<String>,
    chat_history: String,
    file: Option<web::Json<ImageRequest>>,
    api_key: ApiKey,
    user_id: Option<String>,
    session_id: Option<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.unwrap_or_else(|| generate_user_id().to_string());
    let session_id = session_id.unwrap_or_else(|| generate_unique_conversation_id().to_string());

    let chat_history_list: Vec<serde_json::Value> = serde_json::from_str(&chat_history)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid chat history format"))?;

    let response_text = if let Some(text) = text {
        text
    } else {
        file.map_or("Default response".to_string(), |f| f.image_url.clone())
    };

    let response = handle_llm_interaction(response_text, chat_history_list, user_id.clone(), session_id.clone()).await?;

    Ok(HttpResponse::Ok()
        .append_header((SET_COOKIE, format!("user_id={}; Max-Age=31536000", user_id)))
        .append_header((SET_COOKIE, format!("session_id={}; Max-Age=31536000", session_id)))
        .json(response))
}

#[get("/api")]
async fn api_index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "title": "FANA LLM API",
        "description": DESCRIPTION,
        "version": "0.1.0",
        "contact": {
            "name": "FANA AI",
            "url": "https://fana.ai/",
            "email": "hello@fana.ai",
        },
        "license_info": {
            "name": "Apache 2.0",
            "url": "https://www.apache.org/licenses/LICENSE-2.0.html",
        },
    }))
}

#[get("/")]
async fn index() -> impl Responder {
    fs::NamedFile::open_async("./client/index.html").await.unwrap()
}

fn generate_user_id() -> Uuid {
    Uuid::new_v4()
}

fn generate_unique_conversation_id() -> Uuid {
    Uuid::new_v4()
}

async fn handle_llm_interaction(
    response_text: String,
    chat_history_list: Vec<serde_json::Value>,
    user_id: String,
    session_id: String,
) -> Result<LLMResponse, Error> {
    // Simulate handle_llm_interaction function, replace with actual logic
    Ok(LLMResponse {
        response: Some(response_text),
        chat_history: chat_history_list,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    setup_logging().expect("Failed to initialize logging");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(index)
            .service(api_index)
            .service(web::resource("/aimagine/api/v1").route(web::get().to(read_main)))
            .service(web::resource("/aimagine/api/v1/interact-with-llm").route(web::post().to(interact_with_llm)))
            .service(fs::Files::new("/client", "./client").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

