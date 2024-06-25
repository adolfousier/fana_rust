use actix_web::HttpResponse;

pub async fn get_api_key() -> HttpResponse {
    // Mock implementation
    HttpResponse::Ok().finish()
}
