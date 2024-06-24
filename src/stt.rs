use std::env;
use std::fs::File;
use std::io::Read;
use log::info;
use crate::modules::logging_setup::setup_logging;
use async_trait::async_trait;
use std::error::Error;
use reqwest::Client;
use serde_json::Value;

pub struct STTClient {
    client: Client,
    api_key: String,
}

impl STTClient {
    pub fn new(api_key: &str) -> Self {
        let client = Client::new();
        STTClient {
            client,
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
pub trait STT {
    async fn transcribe_audio(
        &self,
        audio_file_path: &str,
        response_format: &str,
        timestamp_granularities: Option<&str>,
    ) -> Result<String, Box<dyn Error>>;

    async fn translate_audio(
        &self,
        audio_file_path: &str,
        target_language: &str,
    ) -> Result<String, Box<dyn Error>>;
}

#[async_trait]
impl STT for STTClient {
    async fn transcribe_audio(
        &self,
        audio_file_path: &str,
        response_format: &str,
        timestamp_granularities: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        let mut audio_file = File::open(audio_file_path)?;
        let mut audio_data = Vec::new();
        audio_file.read_to_end(&mut audio_data)?;

        let form = reqwest::multipart::Form::new()
            .file("file", audio_file_path)?;

        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;

        let transcription: Value = response.json().await?;
        
        Ok(transcription.to_string())
    }

    async fn translate_audio(
        &self,
        audio_file_path: &str,
        target_language: &str,
    ) -> Result<String, Box<dyn Error>> {
        let mut audio_file = File::open(audio_file_path)?;
        let mut audio_data = Vec::new();
        audio_file.read_to_end(&mut audio_data)?;

        let form = reqwest::multipart::Form::new()
            .file("file", audio_file_path)?;

        let response = self.client
            .post("https://api.openai.com/v1/audio/translations")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;

        let translation: Value = response.json().await?;
        
        Ok(translation.to_string())
    }
}

pub fn initialize_logging() {
    setup_logging().expect("Failed to initialize logging");
}

pub async fn create_stt_client() -> STTClient {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    STTClient::new(&api_key)
}

