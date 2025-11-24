use std::fmt;

use nom::error::Error;
use serde::Serialize;

use crate::curl::command::CurlToken;
use crate::curl::parser::curl_cmd_parse;
use crate::curl::url::CurlUrl;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedRequest {
    pub url: CurlUrl,
    pub method: Option<String>,
    pub headers: Vec<String>,
    pub data: Vec<String>,
    pub flags: Vec<String>,
    pub tokens: Vec<CurlToken>,
}

impl ParsedRequest {
    pub fn try_from_tokens(tokens: Vec<CurlToken>) -> Result<Self, RequestBuildError> {
        let mut url: Option<CurlUrl> = None;
        let mut method = None;
        let mut headers = Vec::new();
        let mut data = Vec::new();
        let mut flags = Vec::new();

        for token in &tokens {
            match token {
                CurlToken::Url(parsed_url) => url = Some(parsed_url.clone()),
                CurlToken::Method(field) => method = field.data().map(|value| value.to_string()),
                CurlToken::Header(field) => {
                    if let Some(value) = field.data() {
                        headers.push(value.to_string());
                    }
                }
                CurlToken::Data(field) => {
                    if let Some(value) = field.data() {
                        data.push(value.to_string());
                    }
                }
                CurlToken::Flag(field) => flags.push(field.identifier().to_string()),
            }
        }

        let url = url.ok_or(RequestBuildError::MissingUrl)?;

        Ok(Self {
            url,
            method,
            headers,
            data,
            flags,
            tokens,
        })
    }
}

#[derive(Debug)]
pub enum RequestBuildError {
    MissingUrl,
}

impl fmt::Display for RequestBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestBuildError::MissingUrl => write!(f, "missing target URL"),
        }
    }
}

impl std::error::Error for RequestBuildError {}

#[derive(Debug)]
pub enum ParseError {
    MissingUrl,
    Nom(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingUrl => write!(f, "missing target URL"),
            ParseError::Nom(reason) => write!(f, "nom parser error: {}", reason),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<RequestBuildError> for ParseError {
    fn from(value: RequestBuildError) -> Self {
        match value {
            RequestBuildError::MissingUrl => ParseError::MissingUrl,
        }
    }
}

impl<'a> From<nom::Err<Error<&'a str>>> for ParseError {
    fn from(value: nom::Err<Error<&'a str>>) -> Self {
        ParseError::Nom(value.to_string())
    }
}

pub fn parse_curl_command(input: &str) -> Result<ParsedRequest, ParseError> {
    let (_, tokens) = curl_cmd_parse(input).map_err(ParseError::from)?;
    ParsedRequest::try_from_tokens(tokens).map_err(ParseError::from)
}
