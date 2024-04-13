use crate::text::{byte_index_at_char, char_count};

pub fn insert_char_at_cursor(buf: &mut String, cursor: &mut usize, ch: char) {
    let idx = byte_index_at_char(buf, *cursor);
    buf.insert(idx, ch);
    *cursor += 1;
}
