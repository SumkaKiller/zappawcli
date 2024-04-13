use crate::text::{byte_index_at_char, char_count};

pub fn insert_char_at_cursor(buf: &mut String, cursor: &mut usize, ch: char) {
    let idx = byte_index_at_char(buf, *cursor);
    buf.insert(idx, ch);
    *cursor += 1;
}

pub fn delete_before_cursor(buf: &mut String, cursor: &mut usize) {
    if *cursor == 0 { return; }
    let from = byte_index_at_char(buf, *cursor - 1);
    let to = byte_index_at_char(buf, *cursor);
    buf.drain(from..to);
    *cursor -= 1;
}

pub fn delete_at_cursor(buf: &mut String, cursor: &mut usize) {
    if *cursor >= char_count(buf) { return; }
    let from = byte_index_at_char(buf, *cursor);
    let to = byte_index_at_char(buf, *cursor + 1);
    buf.drain(from..to);
}

pub fn move_cursor_left(cursor: &mut usize) { *cursor = cursor.saturating_sub(1); }
pub fn move_cursor_right(buf: &str, cursor: &mut usize) { if *cursor < char_count(buf) { *cursor += 1; } }

pub fn move_cursor_home(cursor: &mut usize) { *cursor = 0; }
pub fn move_cursor_end(buf: &str, cursor: &mut usize) { *cursor = char_count(buf); }

pub fn delete_prev_word(buf: &mut String, cursor: &mut usize) {
    if *cursor == 0 { return; }
    let chars: Vec<char> = buf.chars().collect();
    let mut pos = *cursor;
    while pos > 0 && chars[pos - 1].is_whitespace() { pos -= 1; }
    while pos > 0 && !chars[pos - 1].is_whitespace() { pos -= 1; }
    let mut new_buf = String::new();
    for (i, ch) in chars.into_iter().enumerate() {
        if i < pos || i >= *cursor { new_buf.push(ch); }
    }
    *buf = new_buf;
    *cursor = pos;
}
