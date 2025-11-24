use nomcurl::{curl::command::CurlToken, parse_curl_command};

#[test]
fn parsed_request_exposes_tokens() {
    let cmd = "curl 'https://example.com' -H 'A:1' --data name=value --insecure";
    let parsed = parse_curl_command(cmd).expect("valid curl");
    assert_eq!(parsed.tokens.len(), 4);
    assert!(matches!(parsed.tokens[1], CurlToken::Header(_)));
    assert!(matches!(parsed.tokens[2], CurlToken::Data(_)));
    assert!(matches!(parsed.tokens[3], CurlToken::Flag(_)));
}
