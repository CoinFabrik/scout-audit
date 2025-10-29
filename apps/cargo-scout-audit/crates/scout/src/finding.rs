use serde_json::Value;
use std::collections::HashSet;
use std::result::Result;
use util::json::{json_to_string_exact, json_to_string_opt};

#[derive(Clone, Debug)]
pub struct Finding {
    value: Value,
}

impl Finding {
    pub fn new(value: Value) -> Finding {
        Finding { value }
    }
    pub fn reason(&self) -> String {
        self.value
            .get("reason")
            .and_then(json_to_string_exact)
            .unwrap_or_default()
    }
    fn code_helper(&self) -> Result<String, ()> {
        let message = self.value.get("message").ok_or(())?;
        let code = message.get("code").ok_or(())?;
        let code = code.get("code").ok_or(())?;
        json_to_string_exact(code).ok_or(())
    }
    pub fn code(&self) -> String {
        self.code_helper().unwrap_or_else(|_| String::new())
    }
    pub fn dashed_code(&self) -> String {
        self.code().replace("_", "-")
    }
    pub fn is_scout_finding(&self, filtered_detectors: &HashSet<String>) -> bool {
        self.reason() == "compiler-message" && filtered_detectors.contains(&self.dashed_code())
    }
    pub fn is_compiler_error(&self) -> bool {
        if self.reason() != "compiler-message" {
            return false;
        }
        self.value
            .get("message")
            .and_then(|x| x.get("level"))
            .and_then(json_to_string_exact)
            .map(|x| x == "error")
            .unwrap_or(false)
    }
    pub fn package(&self) -> String {
        json_to_string_opt(self.value.get("target").and_then(|x| x.get("name"))).unwrap_or_default()
    }
    pub fn package_id(&self) -> String {
        json_to_string_opt(self.value.get("package_id")).unwrap_or_default()
    }
    pub fn krate(&self) -> String {
        self.package().replace("_", "-")
    }
    pub fn json(&self) -> Value {
        self.value.clone()
    }
    pub fn decompose(self) -> Value {
        self.value
    }
    pub fn spans(&self) -> Option<Value> {
        self.value
            .get("message")
            .and_then(|x| x.get("spans"))
            .cloned()
    }
    pub fn message(&self) -> String {
        self.value
            .get("message")
            .and_then(|x| x.get("message"))
            .and_then(json_to_string_exact)
            .unwrap_or_default()
    }
    pub fn rendered(&self) -> String {
        self.value
            .get("message")
            .and_then(|x| x.get("rendered"))
            .and_then(json_to_string_exact)
            .unwrap_or_default()
    }
    pub fn children(&self) -> Option<Value> {
        Some(self.value.get("message")?.get("children")?.clone())
    }
}
