pub const BRAILLE: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn frame(tick: u64) -> &'static str {
    BRAILLE[(tick as usize / 2) % BRAILLE.len()]
}
