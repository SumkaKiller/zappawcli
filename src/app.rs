use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Message {
    pub text: String,
}

#[derive(Copy, Clone)]
pub enum HelpState {
    Closed,
    Open,
}
