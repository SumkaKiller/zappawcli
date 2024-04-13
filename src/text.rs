use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn visible_width(s: &str) -> usize { UnicodeWidthStr::width(s) }
pub fn char_width(c: char) -> usize { UnicodeWidthChar::width(c).unwrap_or(0) }
pub fn char_count(s: &str) -> usize { s.chars().count() }
