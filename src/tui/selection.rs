//! Text selection helpers for the TUI.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharRange {
    pub start: usize,
    pub end: usize,
}

impl CharRange {
    pub fn new(start: usize, end: usize) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn slice(&self, text: &str) -> String {
        if self.is_empty() {
            return String::new();
        }
        let chars: Vec<char> = text.chars().collect();
        let start = self.start.min(chars.len());
        let end = self.end.min(chars.len());
        chars[start..end].iter().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSelection {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl LineSelection {
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        let (a_line, a_col, b_line, b_col) = if (start_line, start_col) <= (end_line, end_col) {
            (start_line, start_col, end_line, end_col)
        } else {
            (end_line, end_col, start_line, start_col)
        };
        Self {
            start_line: a_line,
            start_col: a_col,
            end_line: b_line,
            end_col: b_col,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start_line == self.end_line && self.start_col == self.end_col
    }

    pub fn extract(&self, lines: &[String]) -> String {
        if lines.is_empty() || self.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        let last = lines.len().saturating_sub(1);
        let end_line = self.end_line.min(last);
        let start_line = self.start_line.min(last);

        for (i, line) in lines.iter().enumerate().take(end_line + 1).skip(start_line) {
            let chars: Vec<char> = line.chars().collect();
            let (from, to) = if start_line == end_line {
                (
                    self.start_col.min(chars.len()),
                    self.end_col.min(chars.len()),
                )
            } else if i == start_line {
                (self.start_col.min(chars.len()), chars.len())
            } else if i == end_line {
                (0, self.end_col.min(chars.len()))
            } else {
                (0, chars.len())
            };
            if from < to {
                out.push_str(&chars[from..to].iter().collect::<String>());
            }
            if i < end_line {
                out.push('\n');
            }
        }
        out
    }

    pub fn contains(&self, line: usize, col: usize) -> bool {
        if self.is_empty() {
            return false;
        }
        if line < self.start_line || line > self.end_line {
            return false;
        }
        if self.start_line == self.end_line {
            return col >= self.start_col && col < self.end_col;
        }
        if line == self.start_line {
            return col >= self.start_col;
        }
        if line == self.end_line {
            return col < self.end_col;
        }
        true
    }
}
