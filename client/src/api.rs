use serde::{Deserialize, Serialize};

const DEFAULT_SERVER: &str = "http://localhost:3000";

// ── API DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageResponse {
    pub id: String,
    pub username: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ── HTTP Client ───────────────────────────────────────────────────

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
}

fn base_url(server: &str) -> String {
    let s = server.trim_end_matches('/');
    if s.is_empty() {
        DEFAULT_SERVER.to_string()
    } else {
        s.to_string()
    }
}

/// Register a new account on the server.
pub fn register(server: &str, username: &str, password: &str) -> Result<AuthResponse, String> {
    let url = format!("{}/api/register", base_url(server));
    let resp = client()
        .post(&url)
        .json(&RegisterRequest {
            username: username.to_string(),
            password: password.to_string(),
        })
        .send()
        .map_err(|e| format!("connection failed: {e}"))?;

    if resp.status().is_success() {
        resp.json::<AuthResponse>()
            .map_err(|e| format!("bad response: {e}"))
    } else {
        let err = resp.json::<ErrorResponse>()
            .map(|e| e.error)
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(err)
    }
}

/// Log in to an existing account.
pub fn login(server: &str, username: &str, password: &str) -> Result<AuthResponse, String> {
    let url = format!("{}/api/login", base_url(server));
    let resp = client()
        .post(&url)
        .json(&LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        })
        .send()
        .map_err(|e| format!("connection failed: {e}"))?;

    if resp.status().is_success() {
        resp.json::<AuthResponse>()
            .map_err(|e| format!("bad response: {e}"))
    } else {
        let err = resp.json::<ErrorResponse>()
            .map(|e| e.error)
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(err)
    }
}

/// Send a message to the server.
pub fn send_message(server: &str, token: &str, content: &str) -> Result<MessageResponse, String> {
    let url = format!("{}/api/messages", base_url(server));
    let resp = client()
        .post(&url)
        .bearer_auth(token)
        .json(&SendMessageRequest {
            content: content.to_string(),
        })
        .send()
        .map_err(|e| format!("send failed: {e}"))?;

    if resp.status().is_success() {
        resp.json::<MessageResponse>()
            .map_err(|e| format!("bad response: {e}"))
    } else {
        let err = resp.json::<ErrorResponse>()
            .map(|e| e.error)
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(err)
    }
}

/// Fetch message history from the server.
pub fn fetch_messages(
    server: &str,
    token: &str,
    before: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<MessageResponse>, String> {
    let mut url = format!("{}/api/messages", base_url(server));
    let mut params = Vec::new();
    if let Some(b) = before {
        params.push(format!("before={}", urlencoding(b)));
    }
    if let Some(l) = limit {
        params.push(format!("limit={l}"));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let resp = client()
        .get(&url)
        .bearer_auth(token)
        .send()
        .map_err(|e| format!("fetch failed: {e}"))?;

    if resp.status().is_success() {
        resp.json::<Vec<MessageResponse>>()
            .map_err(|e| format!("bad response: {e}"))
    } else {
        let err = resp.json::<ErrorResponse>()
            .map(|e| e.error)
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(err)
    }
}

/// Minimal percent-encoding for query parameters.
fn urlencoding(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('+', "%2B")
        .replace('&', "%26")
        .replace('=', "%3D")
}
