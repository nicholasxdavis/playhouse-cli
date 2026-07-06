use std::fmt::Display;

pub fn print_json<T: serde::Serialize + ?Sized>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
    );
}

pub fn print_error(message: impl Display, json: bool) -> i32 {
    if json {
        print_json(&serde_json::json!({ "error": message.to_string() }));
    } else {
        eprintln!("[x] {message}");
    }
    1
}
