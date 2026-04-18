use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Message {
    pub text: String,
    pub sender: String,
    pub is_mine: bool,
    pub created: Instant,
    /// Server-side message ID (empty for local-only messages)
    pub remote_id: String,
}

#[derive(Copy, Clone)]
pub enum HelpState {
    Closed,
    Open,
}

pub struct App {
    pub username: String,
    pub server_url: String,
    pub token: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub cursor: usize,
    pub scroll: usize,
    pub started: Instant,
    pub help: HelpState,
    pub help_toggled_at: Instant,
    pub status: String,
    pub status_until: Instant,
    /// Tracks the latest `created_at` seen from the server for polling
    pub last_seen_ts: Option<String>,
}

impl App {
    pub fn new(username: &str, server_url: &str, token: &str) -> Self {
        Self {
            username: username.to_string(),
            server_url: server_url.to_string(),
            token: token.to_string(),
            messages: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll: 0,
            started: Instant::now(),
            help: HelpState::Closed,
            help_toggled_at: Instant::now(),
            status: String::from("Connected"),
            status_until: Instant::now() + Duration::from_secs(3),
            last_seen_ts: None,
        }
    }

    pub fn set_status(&mut self, text: impl Into<String>, dur: Duration) {
        self.status = text.into();
        self.status_until = Instant::now() + dur;
    }

    pub fn help_visible(&self) -> bool {
        matches!(self.help, HelpState::Open)
    }
}
