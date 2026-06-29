use crate::error::ZhihuError;
use serde::Serialize;
use std::process;

#[derive(Debug, Serialize)]
struct ErrorOutput {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i32>,
}

/// Serialize a value to a multi-line JSON string (pretty-printed).
///
/// Extracted from `print_json` so the conversion logic is unit-testable
/// without capturing stdout.
pub(crate) fn to_json_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string_pretty(value)
}

/// Serialize a value to a single-line JSON string (compact).
///
/// Extracted from `print_json_line` so the conversion logic is unit-testable
/// without capturing stdout.
pub(crate) fn to_json_line_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string(value)
}

/// Pretty-print a JSON-serializable value to stdout. Returns `Err` if
/// serialization fails (the caller may then decide to print an error and
/// exit). Returning `Result` instead of exiting internally makes this
/// function testable for both happy and failure paths.
pub fn print_json<T: Serialize>(value: &T) -> Result<(), ZhihuError> {
    match to_json_string(value) {
        Ok(s) => {
            println!("{}", s);
            Ok(())
        }
        Err(e) => Err(ZhihuError::InvalidArgument(format!(
            "JSON serialize failed: {e}"
        ))),
    }
}

/// Single-line print a JSON-serializable value to stdout. Returns `Err`
/// if serialization fails. See [`print_json`] for the rationale behind
/// the `Result` return type.
pub fn print_json_line<T: Serialize>(value: &T) -> Result<(), ZhihuError> {
    match to_json_line_string(value) {
        Ok(s) => {
            println!("{}", s);
            Ok(())
        }
        Err(e) => Err(ZhihuError::InvalidArgument(format!(
            "JSON serialize failed: {e}"
        ))),
    }
}

pub fn print_error(err: &ZhihuError) -> ! {
    eprintln!("{}", format_error_json(err));
    process::exit(1);
}

/// Serialize an error to a single-line JSON string suitable for stderr.
///
/// Currently only `ZhihuError::MissingSecret` produces a non-null `code`
/// field (machine-readable sentinel `20001` for tooling that pattern-matches
/// on it). All other variants omit `code` so consumers can rely on
/// `error` being the only mandatory key.
pub(crate) fn format_error_json(err: &ZhihuError) -> String {
    let code = match err {
        ZhihuError::MissingSecret => Some(20001),
        _ => None,
    };
    let out = ErrorOutput {
        error: err.to_string(),
        code,
    };
    // ErrorOutput is a plain {String, Option<i32>} struct and cannot fail
    // to serialize. Use expect so the (defensive) fallback path is not dead
    // code — we explicitly choose panic-over-fallback here to keep the
    // function total.
    serde_json::to_string(&out)
        .expect("ErrorOutput must always serialize: it contains only String and Option<i32>")
}

#[cfg(test)]
mod tests {
    //! Unit tests for the JSON conversion helpers and the error wire format.

    use super::{format_error_json, print_json, print_json_line, to_json_line_string, to_json_string};
    use crate::error::ZhihuError;
    use reqwest::StatusCode;
    use serde::{Serialize, Serializer};
    use serde_json::{json, Value};

    /// A `Serialize` impl that always errors. Used to exercise the failure
    /// paths of `to_json_string` / `to_json_line_string`, which would
    /// otherwise be unreachable with realistic types.
    struct AlwaysFails;
    impl Serialize for AlwaysFails {
        fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    // ---- to_json_string (pretty) ----

    #[test]
    fn to_json_string_produces_pretty_output() {
        let s = to_json_string(&json!({"a": 1, "b": [2, 3]})).unwrap();
        // Pretty output contains newlines and indentation.
        assert!(s.contains('\n'), "expected multi-line output, got: {s}");
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"][1], 3);
    }

    #[test]
    fn to_json_string_propagates_serialize_error() {
        assert!(to_json_string(&AlwaysFails).is_err());
    }

    // ---- to_json_line_string (compact) ----

    #[test]
    fn to_json_line_string_produces_compact_output() {
        let s = to_json_line_string(&json!({"a": 1, "b": [2, 3]})).unwrap();
        assert!(!s.contains('\n'), "expected single line, got: {s}");
        assert!(!s.contains("  "), "expected no indentation, got: {s}");
    }

    #[test]
    fn to_json_line_string_propagates_serialize_error() {
        assert!(to_json_line_string(&AlwaysFails).is_err());
    }

    // ---- print_json / print_json_line ----

    #[test]
    fn print_json_propagates_serialize_error() {
        let result = print_json(&AlwaysFails);
        match result {
            Err(ZhihuError::InvalidArgument(msg)) => {
                assert!(msg.contains("JSON serialize failed"));
                assert!(msg.contains("intentional test failure"));
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn print_json_line_propagates_serialize_error() {
        let result = print_json_line(&AlwaysFails);
        match result {
            Err(ZhihuError::InvalidArgument(msg)) => {
                assert!(msg.contains("JSON serialize failed"));
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    // ---- format_error_json ----

    // 1. MissingSecret must include the 20001 sentinel.
    #[test]
    fn missing_secret_includes_code_20001() {
        let s = format_error_json(&ZhihuError::MissingSecret);
        let v: Value = serde_json::from_str(&s).expect("output is valid JSON");
        assert_eq!(v["code"], 20001);
    }

    // 2. MissingSecret's user-facing message is preserved verbatim.
    #[test]
    fn missing_secret_includes_error_message() {
        let s = format_error_json(&ZhihuError::MissingSecret);
        let v: Value = serde_json::from_str(&s).expect("output is valid JSON");
        assert_eq!(
            v["error"],
            "Missing access secret. Set ZHIHU_ACCESS_SECRET or run 'zhihu auth set-secret'."
        );
    }

    // 3. Non-MissingSecret variants must OMIT the `code` field (not null).
    #[test]
    fn config_dir_not_found_omits_code_field() {
        let s = format_error_json(&ZhihuError::ConfigDirNotFound);
        let v: Value = serde_json::from_str(&s).expect("output is valid JSON");
        assert!(
            v.get("code").is_none(),
            "expected no `code` key, got: {s}"
        );
    }

    // 4. InvalidArgument preserves the user's argument in `error`.
    #[test]
    fn invalid_argument_preserves_user_input() {
        let s = format_error_json(&ZhihuError::InvalidArgument("bad flag".into()));
        let v: Value = serde_json::from_str(&s).expect("output is valid JSON");
        assert_eq!(v["error"], "Invalid argument: bad flag");
    }

    // 5. The Api variant's status+body show up in the rendered message.
    #[test]
    fn api_error_message_contains_status_and_body() {
        let err = ZhihuError::Api {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: "boom".into(),
        };
        let s = format_error_json(&err);
        let v: Value = serde_json::from_str(&s).expect("output is valid JSON");
        assert_eq!(v["error"], "HTTP 500 Internal Server Error: boom");
        assert!(v.get("code").is_none(), "Api errors must omit `code`");
    }

    // 6. The output is single-line (no embedded newlines) — important for
    //    log scrapers that parse line-by-line.
    #[test]
    fn output_is_single_line() {
        let s = format_error_json(&ZhihuError::MissingSecret);
        assert!(!s.contains('\n'), "expected single line, got: {s:?}");
    }
}
