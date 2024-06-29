use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use reqwest::Client;
use log::{error, debug};
use serde_json::Value;
use std::sync::Mutex;
use crate::input_process::process_user_input;

#[derive(Deserialize)]
struct InteractRequest {
    user_input: String,
}

#[derive(Deserialize)]
struct GenerateImageRequest {
    prompt: String,
}

#[derive(Deserialize)]
struct AnalyzeImageRequest {
    url: String,
}

async fn interact_route(
    interact_req: web::Json<InteractRequest>,
    messages_data: web::Data<Mutex<Vec<Value>>>,
    client: web::Data<Client>,
    groq_api_key: web::Data<String>,
) -> impl Responder {
    let mut messages = messages_data.lock().unwrap();
    let groq_api_key = groq_api_key.get_ref().trim(); // Correctly access and trim the API key
    debug!("Using GROQ API Key in API route: {}", groq_api_key);

    match process_user_input(
        interact_req.user_input.clone(),
        &mut messages,
        &client,
        groq_api_key
    ).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            error!("Failed to process user input: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to process user input: {}", e))
        }
    }
}

async fn generate_image_route(req: web::Json<GenerateImageRequest>) -> impl Responder {
    match crate::image_diffusion::generate_image(&req.prompt).await {
        Ok(url) => HttpResponse::Ok().json(url),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn analyze_image_route(req: web::Json<AnalyzeImageRequest>) -> impl Responder {
    match crate::image_vision::analyze_image(&req.url).await {
        Ok(analysis) => HttpResponse::Ok().json(analysis),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig, groq_api_key: String) {
    cfg.service(
        web::scope("/api")
            .app_data(web::Data::new(Client::new()))
            .app_data(web::Data::new(Mutex::new(Vec::<Value>::new())))
            .app_data(web::Data::new(groq_api_key))
            .route("/interact", web::post().to(interact_route))
            .route("/generate", web::post().to(generate_image_route))
            .route("/analyze", web::post().to(analyze_image_route))
    );
}

