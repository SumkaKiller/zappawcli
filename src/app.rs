use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Message {
    pub text: String,
    pub is_mine: bool,
    pub created: Instant,
}

#[derive(Copy, Clone)]
pub enum HelpState {
    Closed,
    Open,
}

pub struct App {
    pub my_nick: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub cursor: usize,
    pub scroll: usize,
    pub started: Instant,
    pub help: HelpState,
    pub help_toggled_at: Instant,
    pub status: String,
    pub status_until: Instant,
}
