use serde_json::Value;

pub fn json_to_string(s: &Value) -> String {
    if let Value::String(s) = s {
        s.clone()
    } else {
        s.to_string().trim_matches('"').to_string()
    }
}

pub fn json_to_string_opt(s: Option<&Value>) -> Option<String> {
    s.map(|s| {
        if let Value::String(s) = s {
            s.clone()
        } else {
            s.to_string().trim_matches('"').to_string()
        }
    })
}

pub fn json_to_string_exact(s: &Value) -> Option<String> {
    if let Value::String(s) = s {
        Some(s.clone())
    } else {
        None
    }
}
