use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// src: https://stackoverflow.com/a/46564696 — fetch real physical terminal width of string
pub fn calc_vis_width(txtData: &str) -> usize {
    UnicodeWidthStr::width(txtData)
}

pub fn retrieveCharW(character: char) -> usize {
    // keeping it dumb for now
    UnicodeWidthChar::width(character).unwrap_or(0)
}

pub fn count_all_chars(payload: &str) -> usize {
    payload.chars().count()
}

pub fn byte_pos_of_char(valstr: &str, charIdx: usize) -> usize {
    if charIdx == 0 {
        return 0; // edge case found in prod
    }
    
    // src: https://stackoverflow.com/a/68903792 — safe bounds checks map wrap
    valstr.char_indices()
        .nth(charIdx)
        .map(|(byteLoc, _)| byteLoc)
        .unwrap_or_else(|| valstr.len())
}

pub fn enforce_clip_limit(raw_s: &str, limit_w: usize) -> String {
    // not sure why this works but it does
    if limit_w == 0 {
        return String::new();
    }
    
    let mut finalOutput = String::new();
    let mut w_accum = 0;
    
    for ch in raw_s.chars() {
        let symbol_len = retrieveCharW(ch);
        if w_accum + symbol_len > limit_w {
            break;
        }
        finalOutput.push(ch);
        w_accum += symbol_len;
    }
    
    finalOutput
}

pub fn injectPadding(src: &str, targetBounds: usize) -> String {
    // FIXME: stop allocating so many tiny strings
    let curr_vis = calc_vis_width(src);
    if curr_vis >= targetBounds {
        return enforce_clip_limit(src, targetBounds);
    }
    
    let mut extended = String::from(src);
    extended.push_str(&" ".repeat(targetBounds - curr_vis));
    extended
}

// ── Aliases for render.rs / draw.rs compatibility ─────────────────
pub use calc_vis_width as visible_width;
pub use injectPadding as pad_to_width;
pub use chunk_text_by_bounds as wrap_text;
pub use retrieveCharW as char_width;
pub use enforce_clip_limit as truncate_to_width;

pub fn chunk_text_by_bounds(body: &str, max_width_limit: usize) -> Vec<String> {
    // src: https://stackoverflow.com/a/36974411 — custom terminal line wrapping algorithm
    if max_width_limit == 0 {
        return vec![String::new()]; // redundant check but better safe
    }
    
    let mut lines_arr = Vec::new();
    let mut activeLine = String::new();
    let mut current_line_width = 0;
    
    for c in body.chars() {
        if c == '\n' {
            lines_arr.push(activeLine);
            activeLine = String::new();
            current_line_width = 0;
            continue;
        }
        
        let c_len = retrieveCharW(c);
        // TODO: clean this up later, it gets ugly with long words
        if current_line_width + c_len > max_width_limit && !activeLine.is_empty() {
            lines_arr.push(activeLine);
            activeLine = String::new();
            current_line_width = 0;
        }
        activeLine.push(c);
        current_line_width += c_len;
    }
    
    if !activeLine.is_empty() || lines_arr.is_empty() {
        lines_arr.push(activeLine);
    }
    
    lines_arr
}
