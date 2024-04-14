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

        loop {
        let timeout = if has_recent_animation(&app) || app.help_visible() { Duration::from_millis(16) } else { Duration::from_millis(200) };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press { continue; }
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::CONTROL) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        (KeyCode::Char('h'), KeyModifiers::CONTROL) | (KeyCode::F(1), _) => {
                            app.help = match app.help { HelpState::Closed => HelpState::Open, HelpState::Open => HelpState::Closed };
                            app.help_toggled_at = Instant::now();
                            app.set_status(if app.help_visible() { "help opened" } else { "help closed" }, Duration::from_secs(2));
                            needs_redraw = true;
                        }
                        (KeyCode::Enter, _) => {
                            let text = app.input.trim().to_string();
                            if !text.is_empty() {
                                let result = commands::run_command(&mut app, &text);
                                app.set_status(result, Duration::from_secs(2));
                                app.input.clear(); app.cursor = 0; app.scroll = 0;
                            }
                            needs_redraw = true;
                        }
                        (KeyCode::Backspace, _) => { input::delete_before_cursor(&mut app.input, &mut app.cursor); needs_redraw = true; }
                        (KeyCode::Delete, _) => { input::delete_at_cursor(&mut app.input, &mut app.cursor); needs_redraw = true; }
                        (KeyCode::Left, _) => { input::move_cursor_left(&mut app.cursor); needs_redraw = true; }
                        (KeyCode::Right, _) => { input::move_cursor_right(&app.input, &mut app.cursor); needs_redraw = true; }
                        (KeyCode::Home, _) => { input::move_cursor_home(&mut app.cursor); needs_redraw = true; }
                        (KeyCode::End, _) => { input::move_cursor_end(&app.input, &mut app.cursor); needs_redraw = true; }
                        (KeyCode::Up, _) => { app.scroll = app.scroll.saturating_add(2); needs_redraw = true; }
                        (KeyCode::Down, _) => { app.scroll = app.scroll.saturating_sub(2); needs_redraw = true; }
                        (KeyCode::PageUp, _) => { app.scroll = app.scroll.saturating_add(8); needs_redraw = true; }
                        (KeyCode::PageDown, _) => { app.scroll = app.scroll.saturating_sub(8); needs_redraw = true; }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => { needs_redraw = true; }
                _ => {}
            }
        }
        if needs_redraw || has_recent_animation(&app) || app.help_visible() {
            render(&mut stdout, &app)?;
            needs_redraw = false;
        }
    }
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()?;
    Ok(())
}


