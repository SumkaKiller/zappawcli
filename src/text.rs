use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn visible_width(s: &str) -> usize { UnicodeWidthStr::width(s) }
pub fn char_width(c: char) -> usize { UnicodeWidthChar::width(c).unwrap_or(0) }
pub fn char_count(s: &str) -> usize { s.chars().count() }

pub fn byte_index_at_char(s: &str, char_idx: usize) -> usize {
    if char_idx == 0 { return 0; }
    s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or_else(|| s.len())
}

pub fn truncate_to_width(s: &str, max_w: usize) -> String {
    if max_w == 0 { return String::new(); }
    let mut out = String::new();
    let mut curr_w = 0;
    for ch in s.chars() {
        let w = char_width(ch);
        if curr_w + w > max_w { break; }
        out.push(ch);
        curr_w += w;
    }
    out
}

pub fn pad_to_width(s: &str, target: usize) -> String {
    let current = visible_width(s);
    if current >= target { return truncate_to_width(s, target); }
    let mut padded = String::from(s);
    padded.push_str(&" ".repeat(target - current));
    padded
}

pub fn wrap_text(text: &str, max_w: usize) -> Vec<String> {
    if max_w == 0 { return vec![String::new()]; }
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut curr_w = 0;
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(current_line); current_line = String::new(); curr_w = 0; continue;
        }
        let w = char_width(ch);
        if curr_w + w > max_w && !current_line.is_empty() {
            lines.push(current_line); current_line = String::new(); curr_w = 0;
        }
        current_line.push(ch);
        curr_w += w;
    }
    if !current_line.is_empty() || lines.is_empty() { lines.push(current_line); }
    lines
}
