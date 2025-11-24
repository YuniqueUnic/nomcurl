use nomcurl::{
    cli_support::{build_json_value, CurlCommand, JsonField},
    parse_curl_command,
};

#[test]
fn build_json_value_with_keys() {
    let cmd = "curl 'https://example.com' -H 'A:1' --data name=value --insecure";
    let parsed = parse_curl_command(cmd).expect("parsed");
    let value =
        build_json_value(&parsed, None, &[JsonField::Url, JsonField::Headers]).expect("json value");
    assert!(value.get("url").is_some());
    assert!(value.get("headers").is_some());
    assert!(value.get("data").is_none());
}

#[test]
fn build_json_value_part_overrides_keys() {
    let cmd = "curl 'https://example.com' --data name=value";
    let parsed = parse_curl_command(cmd).expect("parsed");
    let value =
        build_json_value(&parsed, Some(CurlCommand::Data), &[JsonField::Url]).expect("json value");
    assert!(value.is_array());
}
