// context_manager.rs
pub mod manage_context {
    pub const MAX_CONTEXT_MESSAGES: usize = 10;

    use crate::session_manager::SessionManager;
    use std::path::PathBuf;
    use serde_json::Value;
    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use futures::io;
    use uuid::Uuid;

    pub struct ContextManager {
        session_manager: SessionManager,
    }

    impl ContextManager {
        pub fn new() -> Self {
            ContextManager {
                session_manager: SessionManager::new(),
            }
        }

        pub async fn add_message(&mut self, session_id: &Uuid, message: Value) {
            if let Some(session) = self.session_manager.get_session(session_id) {
                session.push(message);
                self.save_context(session_id).await.unwrap();
            }
        }

        pub async fn trim_context(&mut self, session_id: &Uuid) {
            if let Some(session) = self.session_manager.get_session(session_id) {
                let len = session.len();
                if len > MAX_CONTEXT_MESSAGES {
                    session.drain(0..len - MAX_CONTEXT_MESSAGES);
                }
                self.save_context(session_id).await.unwrap();
            }
        }

        pub async fn get_context(&mut self, session_id: &Uuid) -> Vec<Value> {
            if let Some(session) = self.session_manager.get_session(session_id) {
                session.clone()
            } else {
                Vec::new()
            }
        }

        pub async fn save_context(&mut self, session_id: &Uuid) -> io::Result<()> {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // Get the directory of the Cargo.toml file
            path.push("memory"); // Add the "memory" directory
            path.push(session_id.to_string()); // Add the session ID
            path.push("context.json"); // Add the file name
            let dir_path = path.parent().unwrap();
            fs::create_dir_all(dir_path).await?;
            if let Some(session) = self.session_manager.get_session(session_id) {
                let json = serde_json::to_string_pretty(&session)?;
                let mut file = fs::File::create(path).await?;
                file.write_all(json.as_bytes()).await?;
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Session not found"))
            }
        }

        pub async fn load_context(&mut self, session_id: &Uuid) -> io::Result<()> {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // Get the directory of the Cargo.toml file
            path.push("memory"); // Add the "memory" directory
            path.push(session_id.to_string()); // Add the session ID
            path.push("context.json"); // Add the file name
            let dir_path = path.parent().unwrap();
            fs::create_dir_all(dir_path).await?;
            if path.exists() {
                let mut file = fs::File::open(path).await?;
                let mut contents = String::new();
                file.read_to_string(&mut contents).await?;
                let session: Vec<Value> = serde_json::from_str(&contents)?;
                let new_session_id = self.session_manager.create_session();
                if let Some(session_mut) = self.session_manager.get_session(&new_session_id) {
                    session_mut.extend(session);
                } else {
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to create new session"));
                }
            }
            Ok(())
        }
    }
}
