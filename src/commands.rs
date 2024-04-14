use std::fs::File;
use std::io::{self, Write};
use std::time::Instant;
use crate::app::{App, HelpState, Message};

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
