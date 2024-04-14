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

fn has_recent_animation(app: &App) -> bool {
    app.messages.last().map(|m| m.created.elapsed() < Duration::from_millis(220)).unwrap_or(false)
}

fn main() -> io::Result<()> {
    let nick = prompt_nickname()?;
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide, terminal::SetTitle(APP_NAME))?;
    let mut app = App::new(&nick);
    let mut needs_redraw = true;
    render(&mut stdout, &app)?;

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()?;
    Ok(())
}


