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

impl App {
    pub fn new(nick: &str) -> Self {
        Self {
            my_nick: nick.to_string(),
            messages: vec![],
            input: String::new(),
            cursor: 0,
            scroll: 0,
            started: Instant::now(),
            help: HelpState::Closed,
            help_toggled_at: Instant::now(),
            status: String::from("Ready"),
            status_until: Instant::now(),
        }
    }
}
