use crossterm::{ cursor, queue, style::{Color, Print, ResetColor, SetForegroundColor} };
use std::io::{self, Stdout};
use crate::text::{char_width, truncate_to_width, visible_width};

pub fn hline(stdout: &mut Stdout, x: u16, y: u16, width: u16) -> io::Result<()> {
    if width < 2 { return Ok(()); }
    queue!(stdout, cursor::MoveTo(x, y), Print("+"))?;
    if width > 2 { queue!(stdout, Print("-".repeat(width.saturating_sub(2) as usize)))?; }
    queue!(stdout, Print("+"))?;
    Ok(())
}

pub fn box_frame(stdout: &mut Stdout, x: u16, y: u16, w: u16, h: u16) -> io::Result<()> {
    if w < 2 || h < 2 { return Ok(()); }
    hline(stdout, x, y, w)?;
    for row in 1..h.saturating_sub(1) {
        queue!(stdout, cursor::MoveTo(x, y + row), Print("|"), cursor::MoveTo(x + w - 1, y + row), Print("|"))?;
    }
    hline(stdout, x, y + h - 1, w)?;
    Ok(())
}

pub fn draw_text(stdout: &mut Stdout, x: u16, y: u16, text: &str, color: Color) -> io::Result<()> {
    queue!(stdout, SetForegroundColor(color), cursor::MoveTo(x, y), Print(text), ResetColor)?;
    Ok(())
}
