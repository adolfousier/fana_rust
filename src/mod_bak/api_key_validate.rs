use std::env;

pub fn validate_user(provided_token: &str) -> bool {
    match env::var("API_KEY") {
        Ok(api_key) => provided_token == api_key,
        Err(_) => false,
    }
}

