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

pub fn bubble_lines(text: &str, max_bubble_w: usize) -> (Vec<String>, usize) {
    let inner_w = max_bubble_w.saturating_sub(4).max(1);
    let wrapped = wrap_text(text, inner_w);
    let mut content_w = 0;
    for line in &wrapped { content_w = content_w.max(visible_width(line)); }
    let bubble_w = (content_w + 4).max(10).min(max_bubble_w.max(10));
    let inner_pad = bubble_w.saturating_sub(4);
    let mut lines = Vec::with_capacity(wrapped.len() + 2);
    lines.push(format!("+{}+", "-".repeat(bubble_w.saturating_sub(2))));
    for line in wrapped {
        let body = pad_to_width(&truncate_to_width(&line, inner_pad), inner_pad);
        lines.push(format!("| {} |", body));
    }
    lines.push(format!("+{}+", "-".repeat(bubble_w.saturating_sub(2))));
    (lines, bubble_w)
}

pub fn layout_message_lines(msg: &Message, term_w: u16, chat_w: u16) -> (Vec<String>, usize, u16, Color, Color) {
    let max_bubble_w = chat_w.saturating_mul(70) as usize / 100;
    let (lines, bubble_w) = bubble_lines(&msg.text, max_bubble_w.max(16));
    let age_ms = msg.created.elapsed().as_millis() as f32;
    let progress = (age_ms / 180.0).clamp(0.0, 1.0);
    let slide = if progress >= 1.0 { 0u16 } else { ((1.0 - progress) * if msg.is_mine { 6.0 } else { 4.0 }) as u16 };
    let frame_color = if msg.is_mine { Color::Cyan } else { Color::Green };
    let bubble_w_u16 = bubble_w.min(term_w.saturating_sub(2) as usize) as u16;
    (lines, bubble_w_u16 as usize, slide, frame_color, Color::White)
}

pub fn help_panel_lines() -> [&'static str; 9] {
    ["Commands", "/help        toggle this panel", "/clear       clear messages", "/nick NAME   change nickname", "/save        save chat to file", "/me TEXT     action message", "/demo        add demo messages", "/top         jump to top", "/bottom      jump to bottom"]
}
pub fn help_panel_progress(app: &App) -> f32 {
    let elapsed = app.help_toggled_at.elapsed().as_millis() as f32;
    (elapsed / 160.0).clamp(0.0, 1.0)
}

pub fn render_help_panel(stdout: &mut Stdout, app: &App, term_w: u16, footer_y: u16, chat_bottom: u16) -> io::Result<u16> {
    if !app.help_visible() && help_panel_progress(app) >= 1.0 { return Ok(0); }
    let lines = help_panel_lines();
    let target_h = lines.len() as u16 + 2;
    let p = help_panel_progress(app);
    let visible_h = if app.help_visible() { (target_h as f32 * p).ceil() as u16 } else { (target_h as f32 * (1.0 - p)).ceil() as u16 }.min(target_h);
    if visible_h < 2 { return Ok(0); }
    let panel_w = term_w.saturating_sub(2).max(18);
    let panel_y = chat_bottom.saturating_sub(visible_h);
    box_frame(stdout, 1, panel_y, panel_w, visible_h)?;
    let max_lines = visible_h.saturating_sub(2) as usize;
    for (i, line) in lines.iter().take(max_lines).enumerate() {
        let y = panel_y + 1 + i as u16;
        let available = panel_w.saturating_sub(4) as usize;
        let clipped = truncate_to_width(line, available);
        let color = if i == 0 { Color::Cyan } else { Color::White };
        draw_text(stdout, 3, y, &clipped, color)?;
    }
    Ok(visible_h)
}

fn command_help_text() -> &'static str { "Commands: /help /clear /nick /save /me /demo /top /bottom" }

pub fn render(stdout: &mut Stdout, app: &App) -> io::Result<()> {
    let (term_w, term_h) = terminal::size()?;
    let term_w = term_w.max(30);
    let term_h = term_h.max(12);

    queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    let header_h = 3;
    let input_h = 3;
    let footer_y = term_h.saturating_sub(input_h);
    let chat_top = header_h;
    let chat_bottom = footer_y;
    let chat_h = chat_bottom.saturating_sub(chat_top);

    box_frame(stdout, 0, 0, term_w, header_h)?;
    draw_centered(stdout, 1, term_w, commands::APP_NAME, Color::White)?;

        let status = if std::time::Instant::now() <= app.status_until { app.status.clone() } else { command_help_text().to_string() };
    let status_w = visible_width(&status) as u16;
    if status_w + 2 < term_w { draw_text(stdout, term_w - status_w - 2, 1, &status, Color::DarkGrey)?; }

    box_frame(stdout, 0, footer_y, term_w, input_h)?;
    Ok(())
}
