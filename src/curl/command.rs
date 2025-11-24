use crate::curl::{config, url::CurlUrl};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurlField {
    pub identifier: String,
    pub data: Option<String>,
}

impl CurlField {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.into(),
            data: None,
        }
    }

    pub fn new_with_data(identifier: &str, data: &str) -> Self {
        Self {
            identifier: identifier.into(),
            data: Some(data.into()),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CurlToken {
    Method(CurlField),
    Url(CurlUrl),
    Header(CurlField),
    Data(CurlField),
    Flag(CurlField),
}

pub use CurlToken as Curl;

impl CurlToken {
    pub fn new(identifier: &str, param: &str) -> Option<Self> {
        if param.trim().is_empty() {
            return None;
        }

        if config::METHOD_FLAG_IDENTIFIERS.contains(&identifier) {
            return Some(CurlToken::Method(CurlField::new_with_data(
                "-X",
                param.trim(),
            )));
        }

        if config::HEADER_FLAG_IDENTIFIERS.contains(&identifier) {
            return Some(CurlToken::Header(CurlField::new_with_data(
                "-H",
                param.trim(),
            )));
        }

        if config::DATA_FLAG_IDENTIFIERS.contains(&identifier) {
            return Some(CurlToken::Data(CurlField::new_with_data("-d", param)));
        }

        None
    }

    pub fn new_flag(identifier: &str) -> Option<Self> {
        Self::new_flag_with_value(identifier, None)
    }

    pub fn new_flag_with_value(identifier: &str, value: Option<&str>) -> Option<Self> {
        let trimmed = identifier.trim();
        if trimmed.is_empty() {
            return None;
        }
        let mut field = CurlField::new(trimmed);
        if let Some(value) = value {
            if !value.trim().is_empty() {
                field.data = Some(value.trim().into());
            }
        }
        Some(CurlToken::Flag(field))
    }

    pub fn new_url(url: CurlUrl) -> Self {
        CurlToken::Url(url)
    }

    pub fn identifier(&self) -> &str {
        match self {
            CurlToken::Method(field)
            | CurlToken::Header(field)
            | CurlToken::Data(field)
            | CurlToken::Flag(field) => field.identifier(),
            CurlToken::Url(_) => "--url",
        }
    }

    pub fn data(&self) -> Option<&str> {
        match self {
            CurlToken::Method(field) | CurlToken::Header(field) | CurlToken::Data(field) => {
                field.data()
            }
            _ => None,
        }
    }

    pub fn expects_value(identifier: &str) -> bool {
        let normalized = identifier.trim();
        config::METHOD_FLAG_IDENTIFIERS.contains(&normalized)
            || config::HEADER_FLAG_IDENTIFIERS.contains(&normalized)
            || config::DATA_FLAG_IDENTIFIERS.contains(&normalized)
    }

    pub fn flag_requires_value(identifier: &str) -> bool {
        let normalized = identifier.trim();
        config::FLAG_VALUE_REQUIRED.contains(&normalized)
            || config::SHORT_FLAGS_VALUE_REQUIRED.contains(&normalized)
    }
}

#[macro_export]
macro_rules! new_curl {
    ($identifier:expr) => {
        $crate::curl::command::CurlToken::new_flag($identifier).expect("invalid flag token")
    };
    ($identifier:expr,$data:expr) => {
        $crate::curl::command::CurlToken::new(stringify!($identifier), $data)
            .expect("invalid curl token")
    };
}
