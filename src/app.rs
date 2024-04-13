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
            messages: vec![
                Message { text: "Привет! Как дела?".to_string(), is_mine: false, created: Instant::now() - Duration::from_millis(300) },
                Message { text: "Отлично, спасибо!".to_string(), is_mine: true, created: Instant::now() - Duration::from_millis(300) },
                Message { text: "Что нового у тебя?".to_string(), is_mine: false, created: Instant::now() - Duration::from_millis(300) },
            ],
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

    pub fn set_status(&mut self, text: impl Into<String>, dur: Duration) {
        self.status = text.into();
        self.status_until = Instant::now() + dur;
    }
