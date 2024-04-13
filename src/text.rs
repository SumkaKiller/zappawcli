use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn visible_width(s: &str) -> usize { UnicodeWidthStr::width(s) }
pub fn char_width(c: char) -> usize { UnicodeWidthChar::width(c).unwrap_or(0) }
pub fn char_count(s: &str) -> usize { s.chars().count() }

pub fn byte_index_at_char(s: &str, char_idx: usize) -> usize {
    if char_idx == 0 { return 0; }
    s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or_else(|| s.len())
}
