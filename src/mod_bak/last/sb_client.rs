use std::env;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use log::{info, error};
use std::error::Error;

pub async fn query_supabase() -> Result<Value, Box<dyn Error>> {
    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let supabase_key = env::var("SUPABASE_KEY").expect("SUPABASE_KEY must be set");
    let supabase_table_name = env::var("SUPABASE_TABLE_NAME").expect("SUPABASE_TABLE_NAME must be set");

    let url = format!("{}/rest/v1/{}?select=*", supabase_url, supabase_table_name);

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_key)?);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", supabase_key))?);

    let client = reqwest::Client::new();
    let response = client.get(&url).headers(headers).send().await?;

    if response.status() != 200 {
        let error_message = response.text().await?;
        error!("Supabase query error: {}", error_message);
        return Err(Box::from(format!("Supabase query error: {}", error_message)));
    }

    let json_response = response.json::<Value>().await?;
    Ok(json_response)
}

