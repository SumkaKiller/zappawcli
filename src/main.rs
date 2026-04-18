mod app;
mod commands;
mod input;
mod text;
mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use app::{App, HelpState};
use ui::render::{prompt_connection_info, render};

// Helper to determine if we should animate at 60fps
fn needs_animation(app: &App) -> bool {
    let recent_message = app.messages
        .last()
        .map(|m| m.created.elapsed() < Duration::from_millis(300))
        .unwrap_or(false);
    
    let help_open_animation = app.help_visible() && ui::render::help_panel_progress(app) < 1.0;
    let help_close_animation = !app.help_visible() && ui::render::help_panel_progress(app) < 1.0;

    recent_message || help_open_animation || help_close_animation
}

pub enum AppEvent {
    Input(Event),
    Tick,
}

fn main() -> io::Result<()> {
    let (nick, room, password) = prompt_connection_info()?;

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&nick, &room, &password);
    let mut needs_redraw = true;

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);
    
    // Background thread for input/network events
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll failed") {
                let evt = event::read().expect("can read events");
                if tx.send(AppEvent::Input(evt)).is_err() {
                    break;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
                last_tick = Instant::now();
            }
        }
    });

    loop {
        // Redraw if triggered or if an animation is actively playing
        if needs_redraw || needs_animation(&app) {
            terminal.draw(|f| render(f, &app))?;
            needs_redraw = false;
        }

        // Throttle poll duration if animated
        let timeout = if needs_animation(&app) {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(250)
        };

        if let Ok(app_event) = rx.recv_timeout(timeout) {
            match app_event {
                AppEvent::Input(Event::Key(key)) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            if !app.input.is_empty() {
                                app.input.clear();
                                app.cursor = 0;
                                needs_redraw = true;
                            } else {
                                break;
                            }
                        }
                        (KeyCode::Char('d'), KeyModifiers::CONTROL) => break,
                        
                        (KeyCode::Esc, KeyModifiers::NONE) => {
                            app.input.clear();
                            app.cursor = 0;
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Char('h'), KeyModifiers::CONTROL) | (KeyCode::F(1), _) => {
                            app.help = match app.help {
                                HelpState::Closed => HelpState::Open,
                                HelpState::Open => HelpState::Closed,
                            };
                            app.help_toggled_at = Instant::now();
                            app.set_status(
                                if app.help_visible() { "Help opened" } else { "Help closed" },
                                Duration::from_secs(2),
                            );
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Enter, _) => {
                            let text = app.input.trim().to_string();
                            if !text.is_empty() {
                                let result = commands::run_command(&mut app, &text);
                                app.set_status(result, Duration::from_secs(2));
                                app.input.clear();
                                app.cursor = 0;
                                app.scroll = 0;
                            }
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Backspace, _) => {
                            input::delete_before_cursor(&mut app.input, &mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Delete, _) => {
                            input::delete_at_cursor(&mut app.input, &mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Left, _) => {
                            input::move_cursor_left(&mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Right, _) => {
                            input::move_cursor_right(&app.input, &mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Home, _) => {
                            input::move_cursor_home(&mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::End, _) => {
                            input::move_cursor_end(&app.input, &mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Up, _) => {
                            app.scroll = app.scroll.saturating_add(2);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Down, _) => {
                            app.scroll = app.scroll.saturating_sub(2);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::PageUp, _) => {
                            app.scroll = app.scroll.saturating_add(8);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::PageDown, _) => {
                            app.scroll = app.scroll.saturating_sub(8);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                            input::move_cursor_home(&mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                            input::move_cursor_end(&app.input, &mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            app.input.clear();
                            app.cursor = 0;
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                            input::delete_prev_word(&mut app.input, &mut app.cursor);
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Tab, _) => {
                            input::insert_char_at_cursor(&mut app.input, &mut app.cursor, ' ');
                            input::insert_char_at_cursor(&mut app.input, &mut app.cursor, ' ');
                            needs_redraw = true;
                        }
                        
                        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                            if !c.is_control() {
                                input::insert_char_at_cursor(&mut app.input, &mut app.cursor, c);
                                needs_redraw = true;
                            }
                        }
                        _ => {}
                    }
                }
                AppEvent::Input(Event::Resize(_, _)) => {
                    needs_redraw = true;
                }
                AppEvent::Input(_) => {}
                AppEvent::Tick => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    Ok(())
}
