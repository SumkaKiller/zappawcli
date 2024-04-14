use crossterm::{ cursor, execute, queue, style::{Color, Print, ResetColor, SetForegroundColor}, terminal::{self, ClearType} };
use std::io::{self, Stdout, Write};
use crate::app::{App, Message};
use crate::commands;
use crate::text::{pad_to_width, truncate_to_width, visible_width, wrap_text};
use crate::ui::draw::{box_frame, draw_centered, draw_text, input_view};

pub fn intro_art() -> [&'static str; 6] {
    [
        r"  _____                  _____        _____ _      ___   ",
        r" |__  /___ _ __ ___     / ____|      / ____| |    |__ \  ",
        r"   / // _ \ '__/ __|   | |  __  __ _| |    | |       ) | ",
        r"  / /|  __/ |  \__ \   | | |_ |/ _` | |    | |      / /  ",
        r" /____\___|_|  |___/   | |__| | (_| | |____| |____ / /_  ",
        r"                         \_____|\__,_|\_____|______|____| ",
    ]
}

pub fn prompt_nickname() -> io::Result<String> {
    let mut stdout = io::stdout();
    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    execute!(stdout, cursor::Hide, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
    let art = intro_art();
    let start_y = term_h.saturating_sub(12) / 2;
    for (i, line) in art.iter().enumerate() {
        let clipped = truncate_to_width(line, term_w as usize);
        let x = term_w.saturating_sub(visible_width(&clipped) as u16) / 2;
        draw_text(&mut stdout, x, start_y + i as u16, &clipped, Color::White)?;
    }
    draw_centered(&mut stdout, start_y + 8, term_w, "clean terminal messenger", Color::DarkGrey)?;
    draw_centered(&mut stdout, start_y + 10, term_w, "enter you nicname: ", Color::White)?;
    stdout.flush()?;
    let mut nick = String::new();
    io::stdin().read_line(&mut nick)?;
    let nick = nick.trim();
    Ok(if nick.is_empty() { "You".to_string() } else { nick.to_string() })
}
