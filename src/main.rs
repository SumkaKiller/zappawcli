mod app;
mod commands;
mod input;
mod text;
mod ui;

use crossterm::{ cursor, event::{self, Event, KeyCode, KeyEventKind, KeyModifiers}, execute, style::ResetColor, terminal };
use std::io;
use std::time::{Duration, Instant};
use app::{App, HelpState};
use commands::APP_NAME;
use ui::render::{prompt_nickname, render};

fn main() {}
