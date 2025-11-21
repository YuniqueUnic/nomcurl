use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Protocol {
    Http,
    Https,
    Ftp,
    Smb,
    Other(String),
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Https
    }
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
            Protocol::Ftp => "ftp",
            Protocol::Smb => "smb",
            Protocol::Other(value) => value.as_str(),
        }
    }
}

impl From<&str> for Protocol {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "http" => Self::Http,
            "https" => Self::Https,
            "ftp" => Self::Ftp,
            "smb" => Self::Smb,
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub password: Option<String>,
}

impl UserInfo {
    pub fn new(username: &str, password: Option<&str>) -> Option<Self> {
        if username.is_empty() {
            return None;
        }

        Some(Self {
            username: username.into(),
            password: password.map(|pwd| pwd.to_string()),
        })
    }

    pub fn from_raw(raw: &str) -> Option<Self> {
        if raw.is_empty() {
            return None;
        }
        let mut parts = raw.splitn(2, ':');
        let username = parts.next().unwrap_or("");
        let password = parts.next();
        Self::new(username, password)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct CurlUrl {
    pub protocol: Protocol,
    pub userinfo: Option<UserInfo>,
    pub domain: String,
    pub uri: Option<String>,
    pub queries: Option<Vec<(String, String)>>,
    pub fragment: Option<String>,
}

impl CurlUrl {
    pub fn new(protocol: &str, domain: &str) -> Self {
        Self {
            protocol: protocol.into(),
            userinfo: None,
            domain: domain.into(),
            uri: None,
            queries: None,
            fragment: None,
        }
    }

    pub fn set_userinfo(&mut self, userinfo: UserInfo) -> &mut Self {
        self.userinfo = Some(userinfo);
        self
    }

    pub fn set_uri(&mut self, uri: &str) -> &mut Self {
        if uri.is_empty() {
            self.uri = None;
        } else {
            self.uri = Some(uri.into());
        }
        self
    }

    pub fn set_queries(&mut self, queries: Vec<(String, String)>) -> &mut Self {
        if queries.is_empty() {
            self.queries = None;
        } else {
            self.queries = Some(queries);
        }
        self
    }

    pub fn set_fragment(&mut self, fragment: &str) -> &mut Self {
        if fragment.is_empty() {
            self.fragment = None;
        } else {
            self.fragment = Some(fragment.into());
        }
        self
    }
}

impl fmt::Display for CurlUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://", self.protocol.as_str())?;
        if let Some(userinfo) = &self.userinfo {
            write!(f, "{}", userinfo.username)?;
            if let Some(password) = &userinfo.password {
                write!(f, ":{}", password)?;
            }
            write!(f, "@")?;
        }
        write!(f, "{}", self.domain)?;
        if let Some(uri) = &self.uri {
            write!(f, "{}", uri)?;
        }
        if let Some(queries) = &self.queries {
            let serialized: Vec<String> = queries
                .iter()
                .map(|(key, value)| format!("{}={}", key, value))
                .collect();
            if !serialized.is_empty() {
                write!(f, "?{}", serialized.join("&"))?;
            }
        }
        if let Some(fragment) = &self.fragment {
            write!(f, "#{}", fragment)?;
        }
        Ok(())
    }
}
