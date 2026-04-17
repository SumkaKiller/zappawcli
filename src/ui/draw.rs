use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Stdout};

use crate::text::{char_width, truncate_to_width, visible_width};

pub fn hline(stdout: &mut Stdout, x: u16, y: u16, width: u16) -> io::Result<()> {
    if width < 2 {
        return Ok(());
    }
    queue!(stdout, cursor::MoveTo(x, y), Print("+"))?;
    if width > 2 {
        let dashes = "-".repeat(width.saturating_sub(2) as usize);
        queue!(stdout, Print(dashes))?;
    }
    queue!(stdout, Print("+"))?;
    Ok(())
}

pub fn box_frame(stdout: &mut Stdout, x: u16, y: u16, w: u16, h: u16) -> io::Result<()> {
    if w < 2 || h < 2 {
        return Ok(());
    }
    hline(stdout, x, y, w)?;
    for row in 1..h.saturating_sub(1) {
        queue!(
            stdout,
            cursor::MoveTo(x, y + row),
            Print("|"),
            cursor::MoveTo(x + w - 1, y + row),
            Print("|")
        )?;
    }
    hline(stdout, x, y + h - 1, w)?;
    Ok(())
}

pub fn draw_text(stdout: &mut Stdout, x: u16, y: u16, text: &str, color: Color) -> io::Result<()> {
    queue!(
        stdout,
        SetForegroundColor(color),
        cursor::MoveTo(x, y),
        Print(text),
        ResetColor
    )?;
    Ok(())
}

pub fn draw_centered(
    stdout: &mut Stdout,
    y: u16,
    term_w: u16,
    text: &str,
    color: Color,
) -> io::Result<()> {
    let clipped = truncate_to_width(text, term_w as usize);
    let x = term_w.saturating_sub(visible_width(&clipped) as u16) / 2;
    draw_text(stdout, x, y, &clipped, color)
}

pub fn input_view(input: &str, cursor: usize, max_w: usize) -> (String, usize) {
    let chars: Vec<char> = input.chars().collect();
    if max_w == 0 {
        return (String::new(), 0);
    }

    let start = cursor.saturating_sub(max_w.saturating_sub(1));
    let mut out = String::new();
    let mut width = 0;
    let mut cursor_x = 0;

    for (i, ch) in chars.iter().enumerate().skip(start) {
        let cw = char_width(*ch);
        if width + cw > max_w {
            break;
        }
        if i < cursor {
            cursor_x += cw;
        }
        out.push(*ch);
        width += cw;
    }

    (out, cursor_x)
}
