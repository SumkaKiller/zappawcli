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
