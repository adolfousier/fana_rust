use azure_sdk_storage_blob::prelude::*;
use azure_sdk_storage_core::key_client::KeyClient;
use azure_sdk_storage_core::Client;
use rand::Rng;
use regex::Regex;
use std::env;
use chrono::Utc;
use std::error::Error;
use std::sync::Mutex;
use futures::StreamExt;
use actix_multipart::Multipart;
use actix_web::web::Bytes;
use uuid::Uuid;
use lazy_static::lazy_static;
use log::{info, error};

lazy_static! {
    static ref STORAGE_CLIENT: Mutex<Option<KeyClient>> = Mutex::new(None);
}

fn get_azure_client() -> Result<KeyClient, Box<dyn Error>> {
    let connection_string = env::var("AZURE_STORAGE_CONNECTION_STRING")
        .expect("AZURE_STORAGE_CONNECTION_STRING must be set");
    let client = KeyClient::from_connection_string(&connection_string)?;
    {
        let mut storage_client = STORAGE_CLIENT.lock().unwrap();
        *storage_client = Some(client.clone());
    }
    Ok(client)
}

async fn generate_random_id(length: usize) -> String {
    info!("Sending request to sanitize filename module to generate a random 16 key ID hash.");
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            char::from_digit(idx as u32, 36).unwrap()
        })
        .collect()
}

async fn sanitize_filename() -> Result<String, Box<dyn Error>> {
    info!("Entering the sanitize filename module.");
    let random_id = generate_random_id(16).await;
    info!("Random 16-character ID hash generated successfully: {}.", random_id);
    let timestamp = Utc::now().format("%Y-%m-%d_%H.%M.%S").to_string();
    let filename = format!("{}_{}.png", timestamp, random_id);
    let re = Regex::new(r#"[/\\?%*:|"<>\s]"#)?;
    let sanitized_name = re.replace_all(&filename, "_").to_string();
    info!("Sanitize filename module generated a filename based on the current date and time successfully, followed by a 16-character ID hash: {}.", sanitized_name);
    Ok(sanitized_name)
}

pub async fn upload_image_to_azure(mut payload: Multipart) -> Result<String, Box<dyn Error>> {
    info!("Inside azure blob module. Uploading image to Azure Blob Storage ");

    let client = get_azure_client()?;
    let container_id = env::var("AZURE_STORAGE_CONTAINER_ID").expect("AZURE_STORAGE_CONTAINER_ID must be set ");
    
    let mut sanitized_name = None;
    let mut data: Option<Bytes> = None;
    while let Some(item) = payload.next().await {
        let mut field = item?;
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            data = Some(chunk);
        }
        sanitized_name = Some(sanitize_filename().await?);
    }

    if let Some(data) = data {
        if let Some(sanitized_name) = sanitized_name {
            let blob_client = client.as_blob_client(&container_id, &sanitized_name);
            let response = blob_client.put_block_blob(&data).await?;
            
            let account_name = env::var("AZURE_STORAGE_ACCOUNT_NAME").expect("AZURE_STORAGE_ACCOUNT_NAME must be set");
            let image_url = format!("https://{}.blob.core.windows.net/{}/{}", account_name, container_id, sanitized_name);
            info!("Leaving Azure Blob; image uploaded to Azure Blob storage successfully; URL: {}", image_url);
            Ok(image_url)
        } else {
            error!("Image filename is not set or is invalid");
            Err(Box::from("Image filename is not set or is invalid"))
        }
    } else {
        error!("No data found in the payload");
        Err(Box::from("No data found in the payload"))
    }
}

