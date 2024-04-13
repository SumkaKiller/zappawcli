use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn visible_width(s: &str) -> usize { UnicodeWidthStr::width(s) }
