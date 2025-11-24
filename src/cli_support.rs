use clap::ValueEnum;
use serde_json::{json, Value};

use crate::ParsedRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CurlCommand {
    Method,
    Header,
    Data,
    Flag,
    Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum JsonField {
    Url,
    Method,
    Headers,
    Data,
    Flags,
    Tokens,
}

pub fn build_json_value(
    parsed: &ParsedRequest,
    part: Option<CurlCommand>,
    keys: &[JsonField],
) -> Result<Value, serde_json::Error> {
    if let Some(part) = part {
        let value = match part {
            CurlCommand::Method => json!(parsed.method),
            CurlCommand::Header => json!(parsed.headers),
            CurlCommand::Data => json!(parsed.data),
            CurlCommand::Flag => json!(parsed.flags),
            CurlCommand::Url => json!(parsed.url),
        };
        return Ok(value);
    }

    if !keys.is_empty() {
        let mut map = serde_json::Map::new();
        for key in keys {
            match key {
                JsonField::Url => {
                    map.insert("url".into(), json!(parsed.url));
                }
                JsonField::Method => {
                    map.insert("method".into(), json!(parsed.method));
                }
                JsonField::Headers => {
                    map.insert("headers".into(), json!(parsed.headers));
                }
                JsonField::Data => {
                    map.insert("data".into(), json!(parsed.data));
                }
                JsonField::Flags => {
                    map.insert("flags".into(), json!(parsed.flags));
                }
                JsonField::Tokens => {
                    map.insert("tokens".into(), json!(parsed.tokens));
                }
            }
        }
        return Ok(Value::Object(map));
    }

    serde_json::to_value(parsed)
}

pub fn format_json(value: &Value, pretty: bool) -> Result<String, serde_json::Error> {
    if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
}

pub fn error_payload(code: &str, message: &str) -> Value {
    json!({
        "code": code,
        "error": message,
    })
}
