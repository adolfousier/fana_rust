use std::collections::HashMap;
use uuid::Uuid;
use serde_json::Value;


pub struct SessionManager {
    sessions: HashMap<Uuid, Vec<Value>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self) -> Uuid {
        let session_id = Uuid::new_v4();
        self.sessions.insert(session_id, Vec::new());
        session_id
    }

    pub fn get_session(&mut self, session_id: &Uuid) -> Option<&mut Vec<Value>> {
        self.sessions.get_mut(session_id)
    }
}
