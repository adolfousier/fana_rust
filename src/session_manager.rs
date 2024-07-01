// session_manager.rs
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::Value;
use std::net::IpAddr;

pub struct SessionManager {
    sessions: HashMap<IpAddr, Uuid>,
    session_data: HashMap<Uuid, Vec<Value>>,
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        SessionManager {
            sessions: self.sessions.clone(),
            session_data: self.session_data.clone(),
        }
    }
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: HashMap::new(),
            session_data: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, ip_addr: IpAddr) -> Uuid {
        if let Some(session_id) = self.sessions.get(&ip_addr) {
            return *session_id;
        }
        let session_id = Uuid::new_v4();
        self.sessions.insert(ip_addr, session_id);
        self.session_data.insert(session_id, Vec::new());
        session_id
    }

    pub fn get_session(&mut self, session_id: &Uuid) -> Option<&mut Vec<Value>> {
        self.session_data.get_mut(session_id)
    }
}
