use serde_json::Value;

/// Default sensitive field keys masked during JSON sanitization.
pub const DEFAULT_SENSITIVE_KEYS: &[&str] = &[
    "password",
    "new_password",
    "token",
    "access_token",
    "refresh_token",
    "credential",
    "id_token",
    "client_secret",
    "secret",
];

/// Default mask placeholder string.
pub const DEFAULT_MASKED_STRING: &str = "******";

/// Recursively masks sensitive fields in a JSON payload using default sensitive keys.
pub fn sanitize_json_value(value: &mut Value) {
    sanitize_json_with_keys(value, DEFAULT_SENSITIVE_KEYS, DEFAULT_MASKED_STRING);
}

/// Recursively masks specified sensitive keys in a JSON payload with a custom mask string.
pub fn sanitize_json_with_keys(value: &mut Value, sensitive_keys: &[&str], mask: &str) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if sensitive_keys.iter().any(|&s| s.eq_ignore_ascii_case(key)) {
                    *val = serde_json::json!(mask);
                } else {
                    sanitize_json_with_keys(val, sensitive_keys, mask);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_json_with_keys(item, sensitive_keys, mask);
            }
        }
        _ => {}
    }
}
