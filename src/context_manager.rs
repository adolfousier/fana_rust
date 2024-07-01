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
    use log::{info};
    use std::net::{IpAddr};

    pub struct ContextManager {
        session_manager: SessionManager,
    }

    impl ContextManager {
        pub fn new() -> Self {
            ContextManager {
                session_manager: SessionManager::new(),
            }
        }
        // Add a new message to a user's session. 
        // It takes an IP address and a message as input, creates a new session if one doesn't exist, and adds the message to the session. 
        // If the session already exists, it simply adds the message to the existing session.
        pub async fn add_message(&mut self, ip_addr: IpAddr, message: Value) {
            info!("Received message from IP address: {}", ip_addr);
            let session_id = self.session_manager.create_session(ip_addr);
            if let Some(session) = self.session_manager.get_session(&session_id) {
                session.push(message);
            } else {
                let new_session_id = self.session_manager.create_session(ip_addr);
                if let Some(session_mut) = self.session_manager.get_session(&new_session_id) {
                    session_mut.push(message);
                    self.save_context(&new_session_id).await.unwrap();
                }
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
        // Save the current state of a user's session to a file during different steps. 
        // It takes a session ID as input, retrieves the corresponding session from the SessionManager, and saves the session to a file in JSON format.
        pub async fn save_context(&mut self, session_id: &Uuid) -> io::Result<()> {
            let mut path = PathBuf::from("src/data"); // Create a "data" directory in your project's root directory
            path.push("user_sessions"); // Add the "user_sessions" directory
            path.push(session_id.to_string()); // Add the session ID
            path.push("context.json"); // Add the file name
            let dir_path = path.parent().unwrap();
            fs::create_dir_all(dir_path).await?;
            
            let ip_addr = IpAddr::V4("95.94.61.253".parse().unwrap()); 
            let session = if let Some(s) = self.session_manager.get_session(session_id) {
                s.clone()
            } else {
                let new_session_id = self.session_manager.create_session(ip_addr);
                self.session_manager.get_session(&new_session_id).unwrap().clone()
            };

            let json = serde_json::to_string_pretty(&session)?;
            let mut file = fs::File::create(path).await?;
            file.write_all(json.as_bytes()).await?;
            Ok(())
        }

        pub async fn load_context(&mut self, session_id: &Uuid) -> io::Result<()> {
            let mut path = PathBuf::from("src/data"); // Create a "data" directory in your project's root directory
            path.push("user_sessions"); // Add the "user_sessions" directory
            path.push(session_id.to_string()); // Add the session ID
            path.push("context.json"); // Add the file name
            let dir_path = path.parent().unwrap();
            fs::create_dir_all(dir_path).await?;
            if path.exists() {
                let mut file = fs::File::open(path).await?;
                let mut contents = String::new();
                file.read_to_string(&mut contents).await?;
                let session: Vec<Value> = serde_json::from_str(&contents)?;
                let ip_addr = IpAddr::V4("95.94.61.253".parse().unwrap()); // Replace with the actual IP address
                let new_session_id = self.session_manager.create_session(ip_addr);
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
