// context_manager.rs
use serde_json::Value;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::io;


pub struct ContextManager {
    session: Vec<Value>,
    memory_dir: String,
}

impl ContextManager {
    pub fn new(memory_dir: &str) -> Self {
        ContextManager {
            session: Vec::new(),
            memory_dir: memory_dir.to_string(),
        }
    }

    pub async fn add_message(&mut self, message: Value) {
        self.session.push(message);
        self.save_context().await.unwrap();
    }

    pub async fn trim_context(&mut self, max_messages: usize) {
        let len = self.session.len();
        if len > max_messages {
            self.session.drain(0..len - max_messages);
        }
        self.save_context().await.unwrap();
    }

    pub fn get_context(&self) -> Vec<Value> {
        self.session.clone()
    }

    pub async fn save_context(&self) -> io::Result<()> {
        let path = Path::new(self.memory_dir.as_str()).join("context.json");
        let dir_path = path.parent().unwrap();
        fs::create_dir_all(dir_path).await?;
        let json = serde_json::to_string_pretty(&self.session)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }

    pub async fn load_context(&mut self) -> io::Result<()> {
        let path = Path::new(self.memory_dir.as_str()).join("context.json");
        let dir_path = path.parent().unwrap();
        fs::create_dir_all(dir_path).await?;
        if path.exists() {
            let mut file = fs::File::open(path).await?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).await?;
            self.session = serde_json::from_str(&contents)?;
        }
        Ok(())
    }
}
