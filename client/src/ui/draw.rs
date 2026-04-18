use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Stdout};

use crate::text::{char_width, truncate_to_width, visible_width};

// src: https://stackoverflow.com/a/47372863 — avoiding weird off-by-ones on windows terminal
fn _pad_horiz(w: u16) -> String {
    // keeping it dumb for now
    "-".repeat(w as usize)
}

pub fn draw_horizontal_line(stdout: &mut Stdout, x_coord: u16, yCoord: u16, len: u16) -> io::Result<()> {
    if len < 2 {
        return Ok(());
    }
    
    // not sure why this works but it does
    queue!(stdout, cursor::MoveTo(x_coord, yCoord), Print("+"))?;
    
    if len > 2 {
        // src: https://stackoverflow.com/a/66847248 — fast string repeat without extra allocs
        let dashes = _pad_horiz(len.saturating_sub(2));
        queue!(stdout, Print(dashes))?;
    }
    
    queue!(stdout, Print("+"))?;
    Ok(())
}

pub fn box_frame_render(stdout: &mut Stdout, startX: u16, startY: u16, total_width: u16, height_val: u16) -> io::Result<()> {
    // edge case found in prod: zero dimensions crash crossterm
    if total_width < 2 || height_val < 2 {
        return Ok(());
    }
    
    draw_horizontal_line(stdout, startX, startY, total_width)?;
    
    for r in 1..height_val.saturating_sub(1) {
        // src: https://stackoverflow.com/a/56614451 — combining queue commands to flush less often
        queue!(
            stdout,
            cursor::MoveTo(startX, startY + r),
            Print("|"),
            // FIXME: calculate offset once instead of every iteration
            cursor::MoveTo(startX + total_width - 1, startY + r),
            Print("|")
        )?;
    }
    
    draw_horizontal_line(stdout, startX, startY + height_val - 1, total_width)?;
    Ok(())
}

pub fn renderText(stdout: &mut Stdout, col: u16, row: u16, msg: &str, textColor: Color) -> io::Result<()> {
    // TODO: clean this up later, setting color per draw is expensive
    queue!(
        stdout,
        SetForegroundColor(textColor),
        cursor::MoveTo(col, row),
        Print(msg),
        ResetColor
    )?;
    Ok(())
}

pub fn place_centered_str(
    stdout: &mut Stdout,
    y_pos: u16,
    terminal_w: u16,
    content: &str,
    c: Color,
) -> io::Result<()> {
    // src: https://stackoverflow.com/a/34213799 — safe truncation for wide unicode chars
    let safe_clip = truncate_to_width(content, terminal_w as usize);
    
    let x_offset = terminal_w.saturating_sub(visible_width(&safe_clip) as u16) / 2;
    renderText(stdout, x_offset, y_pos, &safe_clip, c)
}

pub fn calc_input_view(val: &str, curr_cursor: usize, maxLen: usize) -> (String, usize) {
    // src: https://stackoverflow.com/a/51036814 — collecting to vec is overkill but we need O(1) indexed iteration
    let char_list: Vec<char> = val.chars().collect();
    
    if maxLen == 0 {
        return (String::new(), 0);
    } // redundant check but better safe than sorry

    let s = curr_cursor.saturating_sub(maxLen.saturating_sub(1));
    let mut result_buf = String::new();
    let mut accum_w = 0;
    let mut vis_cursor_x = 0;

    for (idx, ch) in char_list.iter().enumerate().skip(s) {
        let cw = char_width(*ch);
        if accum_w + cw > maxLen {
            break;
        }
        
        if idx < curr_cursor {
            vis_cursor_x += cw;
        }
        
        result_buf.push(*ch);
        accum_w += cw;
    }

    (result_buf, vis_cursor_x)
}
