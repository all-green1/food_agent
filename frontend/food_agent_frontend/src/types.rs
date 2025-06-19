use serde::{Deserialize, Serialize};
use web_sys::{window, Storage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub token: Option<String>,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub is_authenticated: bool,
}

impl Default for AuthContext {
    fn default() -> Self {
        Self {
            token: None,
            user_id: None,
            username: None,
            is_authenticated: false,
        }
    }
}

impl AuthContext {
    const STORAGE_KEY: &'static str = "food_agent_auth";

    // Save auth context to localStorage
    pub fn save_to_storage(&self) {
        if let Some(storage) = Self::get_local_storage() {
            if let Ok(serialized) = serde_json::to_string(self) {
                let _ = storage.set_item(Self::STORAGE_KEY, &serialized);
            }
        }
    }

    // Load auth context from localStorage
    pub fn load_from_storage() -> Self {
        if let Some(storage) = Self::get_local_storage() {
            if let Ok(Some(serialized)) = storage.get_item(Self::STORAGE_KEY) {
                if let Ok(auth_context) = serde_json::from_str::<AuthContext>(&serialized) {
                    return auth_context;
                }
            }
        }
        Self::default()
    }

    // Clear auth context from localStorage
    pub fn clear_from_storage() {
        if let Some(storage) = Self::get_local_storage() {
            let _ = storage.remove_item(Self::STORAGE_KEY);
        }
    }

    // Helper to get localStorage
    fn get_local_storage() -> Option<Storage> {
        window()?.local_storage().ok()?
    }
}

// Chat related types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
}

// Auth types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub user_id: i32,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub access_token: String,
    pub token_type: String,
    pub user_id: i32,
    pub username: String,
}

// Session types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub message: String,
    pub response: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesResponse {
    pub messages: Vec<SessionMessage>,
}

// Google Calendar types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarUrlResponse {
    pub auth_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarStatusResponse {
    pub is_authenticated: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthRequest {
    pub auth_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
}