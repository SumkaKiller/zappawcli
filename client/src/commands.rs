use std::fs::File;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::api;
use crate::app::{App, HelpState, Message};

pub const APP_NAME: &str = "zappawcli";
pub const COMMAND_FILE: &str = "zappawcli_messages.txt";

pub fn save_messages(path: &str, app: &App) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "{}", APP_NAME)?;
    writeln!(file, "user: {}", app.username)?;
    writeln!(file, "---")?;

    for (i, msg) in app.messages.iter().enumerate() {
        writeln!(file, "#{} [{}] {}", i + 1, msg.sender, msg.text)?;
    }

    Ok(())
}

pub fn run_command(app: &mut App, line: &str) -> String {
    let raw = line.trim();
    if !raw.starts_with('/') {
        // Send message via API
        match api::send_message(&app.server_url, &app.token, raw) {
            Ok(resp) => {
                app.messages.push(Message {
                    text: resp.content,
                    sender: resp.username,
                    is_mine: true,
                    created: Instant::now(),
                    remote_id: resp.id,
                });
                String::from("sent")
            }
            Err(e) => format!("send failed: {e}"),
        }
    } else {
        let mut parts = raw.splitn(2, ' ');
        let cmd = parts
            .next()
            .unwrap_or("")
            .trim_start_matches('/')
            .to_lowercase();
        let arg = parts.next().unwrap_or("").trim();

        match cmd.as_str() {
            "help" => {
                app.help = match app.help {
                    HelpState::Closed => HelpState::Open,
                    HelpState::Open => HelpState::Closed,
                };
                app.help_toggled_at = Instant::now();
                if app.help_visible() {
                    String::from("help opened")
                } else {
                    String::from("help closed")
                }
            }
            "clear" => {
                app.messages.clear();
                app.last_seen_ts = None;
                String::from("chat cleared")
            }
            "nick" => {
                if arg.is_empty() {
                    String::from("usage: /nick NAME (local only)")
                } else {
                    app.username = arg.to_string();
                    format!("display name changed to {}", app.username)
                }
            }
            "save" => match save_messages(COMMAND_FILE, app) {
                Ok(()) => format!("saved to {}", COMMAND_FILE),
                Err(e) => format!("save failed: {}", e),
            },
            "me" => {
                if arg.is_empty() {
                    String::from("usage: /me TEXT")
                } else {
                    let action_text = format!("* {} {}", app.username, arg);
                    match api::send_message(&app.server_url, &app.token, &action_text) {
                        Ok(resp) => {
                            app.messages.push(Message {
                                text: resp.content,
                                sender: resp.username,
                                is_mine: true,
                                created: Instant::now(),
                                remote_id: resp.id,
                            });
                            String::from("action sent")
                        }
                        Err(e) => format!("send failed: {e}"),
                    }
                }
            }
            "refresh" => {
                // Force re-fetch all messages
                app.messages.clear();
                app.last_seen_ts = None;
                match api::fetch_messages(&app.server_url, &app.token, None, Some(50)) {
                    Ok(msgs) => {
                        for m in &msgs {
                            app.messages.push(Message {
                                text: m.content.clone(),
                                sender: m.username.clone(),
                                is_mine: m.username == app.username,
                                created: Instant::now(),
                                remote_id: m.id.clone(),
                            });
                        }
                        if let Some(last) = msgs.last() {
                            app.last_seen_ts = Some(last.created_at.clone());
                        }
                        format!("refreshed: {} messages", msgs.len())
                    }
                    Err(e) => format!("refresh failed: {e}"),
                }
            }
            "top" => {
                app.scroll = usize::MAX;
                String::from("jumped to top")
            }
            "bottom" => {
                app.scroll = 0;
                String::from("jumped to bottom")
            }
            "clearinput" => {
                app.input.clear();
                app.cursor = 0;
                String::from("input cleared")
            }
            _ => String::from("unknown command; try /help"),
        }
    }
}
