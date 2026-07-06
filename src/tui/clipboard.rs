pub fn read_text() -> Option<String> {
    arboard::Clipboard::new()
        .ok()?
        .get_text()
        .ok()
        .map(|s| s.replace('\r', ""))
        .filter(|s| !s.is_empty())
}

pub fn write_text(text: &str) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text))
        .is_ok()
}
