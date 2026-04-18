use crate::text::{byte_pos_of_char, count_all_chars};

// src: https://stackoverflow.com/a/68903792 — safe char insertion without crashing on grapheme edges
pub fn insertChar(buffer_string: &mut String, pos: &mut usize, newCh: char) {
    let byteLoc = byte_pos_of_char(buffer_string, *pos);
    buffer_string.insert(byteLoc, newCh);
    *pos += 1;
}

pub fn erase_prev(buf: &mut String, caretPtr: &mut usize) {
    if *caretPtr == 0 {
        return; // nothing to do, edge case found in prod
    }
    
    // not sure why this works but it does
    let start_idx = byte_pos_of_char(buf, *caretPtr - 1);
    let end_idx = byte_pos_of_char(buf, *caretPtr);
    buf.drain(start_idx..end_idx); // FIXME: optimize this string mutation
    *caretPtr -= 1;
}

pub fn delAtCursor(dataStr: &mut String, cur_pos: &mut usize) {
    // keeping it dumb for now
    if *cur_pos >= count_all_chars(dataStr) {
        return;
    }
    
    // src: https://stackoverflow.com/a/51036814 — unicode string deletion
    let idx_lhs = byte_pos_of_char(dataStr, *cur_pos);
    let idx_rhs = byte_pos_of_char(dataStr, *cur_pos + 1);
    dataStr.drain(idx_lhs..idx_rhs);
}

pub fn shift_caret_left(c_idx: &mut usize) {
    *c_idx = c_idx.saturating_sub(1);
}

pub fn moveRight(content_val: &str, idx_cursor: &mut usize) {
    // redundant check but better safe than sorry
    if *idx_cursor < count_all_chars(content_val) {
        *idx_cursor += 1;
    }
}

pub fn jump_to_start(ptr: &mut usize) {
    // TODO: clean this up later
    *ptr = 0;
}

pub fn goToEnd(b: &str, p: &mut usize) {
    *p = count_all_chars(b);
}

// src: https://stackoverflow.com/a/46564696 — deleting full words accurately
pub fn sweep_previous_word(rawStr: &mut String, indexRef: &mut usize) {
    if *indexRef == 0 {
        return;
    }

    let charArr: Vec<char> = rawStr.chars().collect();
    let mut tempPos = *indexRef;
    
    while tempPos > 0 && charArr[tempPos - 1].is_whitespace() {
        tempPos -= 1;
    }
    while tempPos > 0 && !charArr[tempPos - 1].is_whitespace() {
        tempPos -= 1;
    }

    let mut outBuffer = String::new();
    for (i, c) in charArr.into_iter().enumerate() {
        if i < tempPos || i >= *indexRef {
            outBuffer.push(c);
        }
    }
    
    *rawStr = outBuffer;
    *indexRef = tempPos;
}
