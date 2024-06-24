use reqwest::Client;
use std::error::Error;
use std::env;

pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: &str) -> Self {
        let client = Client::new();
        OpenAIClient {
            client,
            api_key: api_key.to_string(),
        }
    }

    async fn post_request(&self, url: &str, body: reqwest::Body) -> Result<String, Box<dyn Error>> {
        let response = self.client
            .post(url)
            .bearer_auth(&self.api_key)
            .body(body)
            .send()
            .await?;
            
        let result = response.text().await?;
        Ok(result)
    }
}

