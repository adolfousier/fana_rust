use log::{info, error};
use reqwest::Client;
use std::env;
use crate::modules::logging_setup::setup_logging;
use ndarray::Array1;

lazy_static::lazy_static! {
    static ref OPENAI_API_KEY: String = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
}



pub async fn generate_embedding(query_text: &str) -> Result<Vec<f32>, reqwest::Error> {
    info!("Entering generate embedding module.");

    let client = Client::new();
    let response = client.post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", *OPENAI_API_KEY))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "input": query_text,
            "model": "text-embedding-3-small"
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let embedding = json["data"][0]["embedding"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect::<Vec<f32>>();
        
        info!("Embedding generated successfully.");
        Ok(embedding)
    } else {
        let error_message = format!("Failed to generate embedding: {}", response.status());
        error!("{}", error_message);
        Err(reqwest::Error::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR, error_message))
    }
}

pub async fn cosine_similarity(vec_a: Vec<f32>, vec_b: Vec<f32>) -> Result<f32, Box<dyn std::error::Error>> {
    if vec_a.len() != vec_b.len() {
        return Err("Vectors must be of the same length".into());
    }

    let dot_product = vec_a.iter().zip(&vec_b).map(|(x, y)| x * y).sum::<f32>();
    let norm_a = vec_a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let norm_b = vec_b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let similarity = dot_product / (norm_a * norm_b);

    info!("Cosine similarity calculated successfully.");
    Ok(similarity)
}

