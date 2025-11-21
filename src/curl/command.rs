use crate::curl::url::CurlUrl;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurlToken {
    Method(CurlField),
    Url(CurlUrl),
    Header(CurlField),
    Data(CurlField),
    Flag(CurlField),
}

pub use CurlToken as Curl;

impl CurlToken {
    const METHOD_FLAGS: [&'static str; 2] = ["-X", "--request"];
    const HEADER_FLAGS: [&'static str; 2] = ["-H", "--header"];
    const DATA_FLAGS: [&'static str; 5] = [
        "-d",
        "--data",
        "--data-raw",
        "--data-binary",
        "--data-urlencode",
    ];

    pub fn new(identifier: &str, param: &str) -> Option<Self> {
        if param.trim().is_empty() {
            return None;
        }

        if Self::METHOD_FLAGS.contains(&identifier) {
            return Some(CurlToken::Method(CurlField::new_with_data(
                "-X",
                param.trim(),
            )));
        }

        if Self::HEADER_FLAGS.contains(&identifier) {
            return Some(CurlToken::Header(CurlField::new_with_data(
                "-H",
                param.trim(),
            )));
        }

        if Self::DATA_FLAGS.contains(&identifier) {
            return Some(CurlToken::Data(CurlField::new_with_data("-d", param)));
        }

        None
    }

    pub fn new_flag(identifier: &str) -> Option<Self> {
        let trimmed = identifier.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(CurlToken::Flag(CurlField::new(trimmed)))
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
        Self::METHOD_FLAGS.contains(&normalized)
            || Self::HEADER_FLAGS.contains(&normalized)
            || Self::DATA_FLAGS.contains(&normalized)
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
