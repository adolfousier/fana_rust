// api_routes.rs
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/generate_image", web::post().to(generate_image_route))
            .route("/analyze_image", web::post().to(analyze_image_route)),
    );
}

