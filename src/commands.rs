use std::fs::File;
use std::io::{self, Write};
use std::time::Instant;
use crate::app::{App, HelpState, Message};

pub fn run_command(app: &mut App, line: &str) -> String {
    let raw = line.trim();
    if !raw.starts_with('/') {
        app.messages.push(Message { text: raw.to_string(), is_mine: true, created: Instant::now() });
        return String::from("sent");
    }
        let mut parts = raw.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("").trim_start_matches('/').to_lowercase();
    let arg = parts.next().unwrap_or("").trim();

    match cmd.as_str() {
                "help" => {
            app.help = match app.help { HelpState::Closed => HelpState::Open, HelpState::Open => HelpState::Closed, };
            app.help_toggled_at = Instant::now();
            if app.help_visible() { String::from("help opened") } else { String::from("help closed") }
        }
                "clear" => { app.messages.clear(); String::from("chat cleared") }
        _ => String::from("unknown command; try /help"),
    }
}

pub const APP_NAME: &str = "zappawcli";
pub const COMMAND_FILE: &str = "zappawcli_messages.txt";

pub fn save_messages(path: &str, app: &App) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "{}", APP_NAME)?;
    writeln!(file, "nick: {}", app.my_nick)?;
    writeln!(file, "---")?;
    for (i, msg) in app.messages.iter().enumerate() {
        let sender = if msg.is_mine { &app.my_nick } else { "peer" };
        writeln!(file, "#{} [{}] {}", i + 1, sender, msg.text)?;
    }
    Ok(())
}
