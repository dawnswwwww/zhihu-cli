use crate::error::ZhihuError;
use serde::Serialize;
use std::process;

#[derive(Debug, Serialize)]
struct ErrorOutput {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i32>,
}

pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{}", s),
        Err(e) => print_error(
            &ZhihuError::InvalidArgument(format!("JSON serialize failed: {e}")),
        ),
    }
}

pub fn print_json_line<T: Serialize>(value: &T) {
    match serde_json::to_string(value) {
        Ok(s) => println!("{}", s),
        Err(e) => print_error(
            &ZhihuError::InvalidArgument(format!("JSON serialize failed: {e}")),
        ),
    }
}

pub fn print_error(err: &ZhihuError) -> ! {
    let code = match err {
        ZhihuError::MissingSecret => Some(20001),
        _ => None,
    };
    let out = ErrorOutput {
        error: err.to_string(),
        code,
    };
    eprintln!("{}", serde_json::to_string(&out).unwrap_or_else(|_| {
        r#"{"error":"Failed to serialize error"}"#.to_string()
    }));
    process::exit(1);
}
