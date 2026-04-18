mod api;
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

use app::{App, HelpState, Message};
use ui::render::{help_panel_progress, prompt_auth_info, render};

// Helper to determine if we should animate at 60fps
fn needs_animation(app: &App) -> bool {
    let recent_message = app
        .messages
        .last()
        .map(|m| m.created.elapsed() < Duration::from_millis(300))
        .unwrap_or(false);

    let help_open_animation = app.help_visible() && help_panel_progress(app) < 1.0;
    let help_close_animation = !app.help_visible() && help_panel_progress(app) < 1.0;

    recent_message || help_open_animation || help_close_animation
}

pub enum AppEvent {
    Input(Event),
    Tick,
    /// New messages fetched from server
    ServerMessages(Vec<api::MessageResponse>),
}

fn main() -> io::Result<()> {
    // ── Auth prompt ───────────────────────────────────────────
    let auth_info = prompt_auth_info()?;

    // ── Authenticate ──────────────────────────────────────────
    let auth_result = if auth_info.is_register {
        api::register(&auth_info.server_url, &auth_info.username, &auth_info.password)
    } else {
        api::login(&auth_info.server_url, &auth_info.username, &auth_info.password)
    };

    let auth_resp = match auth_result {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("\nAuthentication failed: {e}");
            eprintln!("Press Enter to exit...");
            let mut buf = String::new();
            let _ = io::stdin().read_line(&mut buf);
            return Ok(());
        }
    };

    // ── Enter TUI ─────────────────────────────────────────────
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&auth_resp.username, &auth_info.server_url, &auth_resp.token);
    app.set_status(
        if auth_info.is_register {
            "Registered successfully!"
        } else {
            "Logged in successfully!"
        },
        Duration::from_secs(3),
    );

    let mut needs_redraw = true;

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);

    // ── Background thread: input events ───────────────────────
    let tx_input = tx.clone();
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll failed") {
                let evt = event::read().expect("can read events");
                if tx_input.send(AppEvent::Input(evt)).is_err() {
                    break;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if tx_input.send(AppEvent::Tick).is_err() {
                    break;
                }
                last_tick = Instant::now();
            }
        }
    });

    // ── Background thread: poll server for messages ───────────
    let tx_poll = tx.clone();
    let poll_server = auth_info.server_url.clone();
    let poll_token = auth_resp.token.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(2));

            match api::fetch_messages(&poll_server, &poll_token, None, Some(100)) {
                Ok(msgs) => {
                    if tx_poll.send(AppEvent::ServerMessages(msgs)).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    // Silently retry on next tick
                }
            }
        }
    });

    // ── Initial message fetch ─────────────────────────────────
    if let Ok(msgs) = api::fetch_messages(&auth_info.server_url, &auth_resp.token, None, Some(50)) {
        for m in &msgs {
            app.messages.push(Message {
                text: m.content.clone(),
                sender: m.username.clone(),
                is_mine: m.username == app.username,
                created: Instant::now() - Duration::from_millis(500), // render instantly
                remote_id: m.id.clone(),
            });
        }
        if let Some(last) = msgs.last() {
            app.last_seen_ts = Some(last.created_at.clone());
        }
    }

    // ── Main event loop ───────────────────────────────────────
    loop {
        if needs_redraw || needs_animation(&app) {
            terminal.draw(|f| render(f, &app))?;
            needs_redraw = false;
        }

        let timeout = if needs_animation(&app) {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(250)
        };

        if let Ok(app_event) = rx.recv_timeout(timeout) {
            match app_event {
                AppEvent::ServerMessages(msgs) => {
                    // Merge server messages — only add ones we haven't seen
                    let mut new_count = 0;
                    for m in &msgs {
                        let already_have = app.messages.iter().any(|existing| existing.remote_id == m.id);
                        if !already_have {
                            app.messages.push(Message {
                                text: m.content.clone(),
                                sender: m.username.clone(),
                                is_mine: m.username == app.username,
                                created: Instant::now(),
                                remote_id: m.id.clone(),
                            });
                            new_count += 1;
                        }
                    }
                    if let Some(last) = msgs.last() {
                        app.last_seen_ts = Some(last.created_at.clone());
                    }
                    if new_count > 0 {
                        needs_redraw = true;
                    }
                }
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
                                if app.help_visible() {
                                    "Help opened"
                                } else {
                                    "Help closed"
                                },
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
                            input::erase_prev(&mut app.input, &mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Delete, _) => {
                            input::delAtCursor(&mut app.input, &mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Left, _) => {
                            input::shift_caret_left(&mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Right, _) => {
                            input::moveRight(&app.input, &mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Home, _) => {
                            input::jump_to_start(&mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::End, _) => {
                            input::goToEnd(&app.input, &mut app.cursor);
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
                            input::jump_to_start(&mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                            input::goToEnd(&app.input, &mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            app.input.clear();
                            app.cursor = 0;
                            needs_redraw = true;
                        }

                        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                            input::sweep_previous_word(&mut app.input, &mut app.cursor);
                            needs_redraw = true;
                        }

                        (KeyCode::Tab, _) => {
                            input::insertChar(&mut app.input, &mut app.cursor, ' ');
                            input::insertChar(&mut app.input, &mut app.cursor, ' ');
                            needs_redraw = true;
                        }

                        (KeyCode::Char(c), KeyModifiers::NONE)
                        | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                            if !c.is_control() {
                                input::insertChar(&mut app.input, &mut app.cursor, c);
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
