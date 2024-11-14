use serde_json::Value;
use std::result::Result;
use std::collections::HashSet;
use crate::utils::json::{json_to_string_exact, json_to_string_opt};

#[derive(Clone, Debug)]
pub struct Finding{
    value: Value,
}

impl Finding{
    pub fn new(value: Value) -> Finding{
        Finding{
            value,
        }
    }
    pub fn reason(&self) -> String{
        self.value.get("reason")
            .and_then(|x| json_to_string_exact(x))
            .unwrap_or_else(|| String::new())
    }
    fn code_helper(&self) -> Result<String, ()>{
        let message = self.value.get("message").ok_or(())?;
        let code = message.get("code").ok_or(())?;
        let code = code.get("code").ok_or(())?;
        json_to_string_exact(code).ok_or(())
    }
    pub fn code(&self) -> String{
        self.code_helper()
            .unwrap_or_else(|_| String::new())
    }
    pub fn dashed_code(&self) -> String{
        self.code().replace("_", "-")
    }
    pub fn is_scout_finding(&self, filtered_detectors: &HashSet<String>) -> bool{
        self.reason() == "compiler-message" && filtered_detectors.contains(&self.dashed_code())
    }
    pub fn is_compiler_error(&self) -> bool{
        if self.reason() != "compiler-message"{
            return false;
        }
        self.value
            .get("message")
            .and_then(|x| x.get("level"))
            .and_then(|x| json_to_string_exact(x))
            .and_then(|x| Some(x == "error"))
            .unwrap_or(false)
    }
    pub fn package(&self) -> String{
        json_to_string_opt(self.value.get("target").and_then(|x| x.get("name")))
            .unwrap_or_else(|| String::new())
    }
    pub fn krate(&self) -> String{
        self.package().replace("_", "-")
    }
    pub fn json(&self) -> Value{
        self.value.clone()
    }
    pub fn decompose(self) -> Value{
        self.value
    }
    pub fn spans(&self) -> Option<Value>{
        self.value
            .get("message")
            .and_then(|x| x.get("spans"))
            .and_then(|x| Some(x.clone()))
    }
    pub fn message(&self) -> String{
        self.value
            .get("message")
            .and_then(|x| x.get("message"))
            .and_then(|x| json_to_string_exact(x))
            .unwrap_or_else(|| String::new())
    }
    pub fn rendered(&self) -> String{
        self.value
            .get("message")
            .and_then(|x| x.get("rendered"))
            .and_then(|x| json_to_string_exact(x))
            .unwrap_or_else(|| String::new())
    }
}
