// api_routes.rs
use crate::system_prompt::SYSTEM_PROMPT;
use crate::session_manager::SessionManager;
use crate::input_process::process_user_input;

use actix_web::{web, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use serde::Deserialize;
use reqwest::Client;
use log::{error, debug, info};
use std::sync::{Arc, Mutex};
use serde_json::json;
use std::net::IpAddr;

#[derive(Deserialize)]
struct InteractRequest {
    question: String,
}

// Set API Routes
pub fn configure(cfg: &mut web::ServiceConfig, groq_api_key: web::Data<String>) {
    cfg.service(
        web::scope("/api")
           .app_data(web::Data::new(Client::new()))
           .app_data(web::Data::new(groq_api_key.clone()))
           .route("/interact", web::post().to(interact_route))
    );
}

// Set API Endpoint
async fn interact_route(
    interact_req: web::Json<InteractRequest>,
    client: web::Data<Client>,
    groq_api_key: web::Data<String>, 
    session_manager: web::Data<Arc<Mutex<SessionManager>>>,
) -> impl Responder {
    let session_manager_arc = session_manager.clone();
    let mut session_manager_lock = session_manager_arc.lock().unwrap();
    let ip_addr = IpAddr::V4("95.94.61.253".parse().unwrap());
    let session_id = session_manager_lock.create_session(ip_addr);
    info!("Added assistant message to context for session {}", session_id);
    let mut session = session_manager_lock.get_session(&session_id).unwrap().clone();

    let groq_api_key = groq_api_key.get_ref().trim();

    // Add system prompt if m
    // Messages are empty (first call)
    if session.is_empty() {
        session.push(json!({
            "role": "system",
            "content": SYSTEM_PROMPT
        }));
    }
    debug!("Using GROQ API Key in API route: {}", groq_api_key);

    match process_user_input(
        interact_req.question.clone(),
        &mut session_manager_lock,
        &client,
        groq_api_key,
        ip_addr 
    ).await {
        Ok(response) => {
            // Return the response as plain text
            HttpResponse::Ok()
     .content_type(ContentType::plaintext())
     .body(response)
        }
        Err(e) => {
            error!("Failed to process user input: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to process user input: {}", e))
        }
    }
}


// #[derive(Deserialize)]
// struct GenerateImageRequest {
//     prompt: String,
// }

// #[derive(Deserialize)]
// struct AnalyzeImageRequest {
//     url: String,
// }


// async fn generate_image_route(req: web::Json<GenerateImageRequest>) -> impl Responder {
//     match crate::image_diffusion::generate_image(&req.prompt).await {
//         Ok(url) => HttpResponse::Ok().json(url),
//         Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
//     }
// }

// async fn analyze_image_route(req: web::Json<AnalyzeImageRequest>) -> impl Responder {
//    match crate::image_vision::analyze_image(&req.url).await {
//        Ok(analysis) => HttpResponse::Ok().json(analysis),
//        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
//    }
// }

// pub fn configure(cfg: &mut web::ServiceConfig, groq_api_key: String) {
//     cfg.service(
//         web::scope("/api")
//             .app_data(web::Data::new(Client::new()))
//             .app_data(web::Data::new(Mutex::new(Vec::<Value>::new())))
//             .app_data(web::Data::new(groq_api_key))
//             .route("/interact", web::post().to(interact_route))
//             .route("/generate", web::post().to(generate_image_route))
//             .route("/analyze", web::post().to(analyze_image_route))
//     );
// }


