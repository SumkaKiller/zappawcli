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
