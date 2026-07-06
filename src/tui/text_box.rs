use unicode_width::UnicodeWidthStr;

pub fn width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

pub fn truncate(s: &str, max_width: usize) -> String {
    if width(s) <= max_width {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    let limit = max_width.saturating_sub(3);
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > limit {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push_str("...");
    out
}

pub fn wrap_text(s: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![s.to_string()];
    }
    let mut out = Vec::new();
    for paragraph in s.split('\n') {
        if paragraph.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut line = String::new();
        let mut line_w = 0usize;
        for word in paragraph.split_whitespace() {
            let ww = width(word);
            let extra = if line.is_empty() { ww } else { ww + 1 };
            if !line.is_empty() && line_w + extra > max_width {
                out.push(line);
                line = word.to_string();
                line_w = ww;
            } else if line.is_empty() {
                line = word.to_string();
                line_w = ww;
            } else {
                line.push(' ');
                line.push_str(word);
                line_w += extra;
            }
        }
        out.push(line);
    }
    out
}

pub fn wrap_paragraph_with_starts(s: &str, max_width: usize) -> Vec<(usize, String)> {
    if max_width == 0 {
        return vec![(0, s.to_string())];
    }
    let mut out = Vec::new();
    if s.is_empty() {
        return vec![(0, String::new())];
    }
    let mut line = String::new();
    let mut line_w = 0usize;
    let mut line_start = 0usize;
    let mut i = 0usize;
    let chars: Vec<char> = s.chars().collect();
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let word_start = i;
        let mut word = String::new();
        while i < chars.len() && !chars[i].is_whitespace() {
            word.push(chars[i]);
            i += 1;
        }
        let ww = width(&word);
        let extra = if line.is_empty() { ww } else { ww + 1 };
        if !line.is_empty() && line_w + extra > max_width {
            out.push((line_start, line));
            line = word;
            line_w = ww;
            line_start = word_start;
        } else if line.is_empty() {
            line = word;
            line_w = ww;
            line_start = word_start;
        } else {
            line.push(' ');
            line.push_str(&word);
            line_w += extra;
        }
    }
    out.push((line_start, line));
    out
}

pub fn input_visual_layout(text: &str, max_width: usize) -> Vec<(usize, String)> {
    let mut rows = Vec::new();
    if text.is_empty() {
        return rows;
    }
    let mut char_idx = 0usize;
    let parts: Vec<&str> = text.split('\n').collect();
    for (pi, para) in parts.iter().enumerate() {
        let para_start = char_idx;
        for (local_off, line) in wrap_paragraph_with_starts(para, max_width) {
            rows.push((para_start + local_off, line));
        }
        char_idx = para_start + para.chars().count();
        if pi + 1 < parts.len() {
            char_idx += 1;
        }
    }
    rows
}
