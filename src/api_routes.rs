use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use crate::{process_user_input};
use crate::json;
use reqwest::Client;
use futures::TryStreamExt;
use log::{error, info, debug};

async fn interact(
    mut payload: Multipart,
    client: web::Data<Client>,
    groq_api_key: web::Data<String>,
    system_prompt: web::Data<String>,
) -> impl Responder {
    let mut input = None;
    let mut file_data = Vec::new();

    info!("Processing multipart payload");
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        if let Some(name) = content_disposition.get_name() {
            if name == "input" {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    input = Some(String::from_utf8_lossy(&data).to_string());
                    debug!("Received input: {}", input.as_ref().unwrap());
                }
            } else if name == "file" {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    file_data.extend_from_slice(&data);
                }
                debug!("Received file data");
            }
        }
    }

    let mut messages = vec![
        json!({
            "role": "system",
            "content": system_prompt.as_str()
        })
    ];

    let user_input = input.or_else(|| {
        if !file_data.is_empty() {
            Some(String::from_utf8_lossy(&file_data).to_string())
        } else {
            None
        }
    }).unwrap_or_default();

    info!("User input: {}", user_input);

    match process_user_input(user_input, &mut messages, &client, &groq_api_key).await {
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
        web::scope("/api/prediction")
            .route("/interact", web::post().to(interact)),
    );
}

