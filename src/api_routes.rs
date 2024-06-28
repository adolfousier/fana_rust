use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use reqwest::Client;
use log::{error, info, debug};

#[derive(Deserialize)]
struct InteractRequest {
    text: String,
}

#[derive(Deserialize)]
struct GenerateImageRequest {
    prompt: String,
}

#[derive(Deserialize)]
struct AnalyzeImageRequest {
    url: String,
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

async fn interact_route(
    req: web::Json<InteractRequest>,
    client: web::Data<Client>,
    groq_api_key: web::Data<String>,
    system_prompt: web::Data<String>,
) -> impl Responder {
    let mut messages = vec![
        json!({
            "role": "system",
            "content": system_prompt.as_str()
        })
    ];

    debug!("Received text: {}", req.text);

    match crate::input_process::process_user_input(
        req.text.clone(),
        &mut messages,
        &client,
        groq_api_key.as_str(),
    ).await {
        Ok(response) => {
            info!("Fana response: {}", response);
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            error!("Error processing user input: {}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/interact", web::post().to(interact_route))
            .route("/generate", web::post().to(generate_image_route))
            .route("/analyze", web::post().to(analyze_image_route))
    );
}

