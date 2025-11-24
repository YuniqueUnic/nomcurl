use nomcurl::{
    curl::parser::{
        credentials_domain_parse, credentials_domain_to_host_parse,
        credentials_domain_to_userinfo_parse, curl_url_parse, fragment_parse, protocol_parse,
        queries_parse, queries_to_query_fragments, uri_parse, uri_to_path_fragments,
    },
    curl::CurlUrl,
    test_util::{generic_command_parse, generic_parse},
};

const TEST_URL_FULL: &str =
    "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open#ABC";

#[test]
fn test_curl_url_parse() {
    let input = TEST_URL_FULL;
    let userinfo = nomcurl::curl::url::UserInfo::from_raw("user:passwd").unwrap();
    let queries = queries_to_query_fragments("?labels=E-easy&state=open");
    let mut expect = CurlUrl::new("https", "github.com");
    expect
        .set_userinfo(userinfo)
        .set_uri("/rust-lang/rust/issues")
        .set_queries(queries)
        .set_fragment("ABC");
    generic_command_parse(curl_url_parse, input, expect.clone());

    let input = "http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498";
    let queries = queries_to_query_fragments("?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498");
    let mut expect = CurlUrl::new("http", "query.sse.com.cn");
    expect.set_uri("/commonQuery.do").set_queries(queries);
    generic_command_parse(curl_url_parse, input, expect);
}

#[test]
fn test_protocol_parse() {
    let input = TEST_URL_FULL;
    generic_command_parse(protocol_parse, input, "https".into());
    let input = input.replace("https", "smb");
    generic_command_parse(protocol_parse, &input, "smb".into());
    let input = input.replace("smb", "FTP");
    generic_command_parse(protocol_parse, &input, "FTP".into());
}

#[test]
fn test_credentials_domain_parse() {
    let input = TEST_URL_FULL.replace("https://", "");
    generic_command_parse(credentials_domain_parse, &input, "user:passwd@github.com");
}

#[test]
fn test_credentials_domain_to_userinfo_parse() {
    generic_command_parse(
        credentials_domain_to_userinfo_parse,
        "user:passwd@github.com",
        "user:passwd",
    );
}

#[test]
fn test_credentials_domain_to_host_parse() {
    let (_, host) = credentials_domain_to_host_parse("user:passwd@github.com").expect("host");
    assert_eq!(host, "github.com");
}

#[test]
fn test_uri_parse() {
    let input = TEST_URL_FULL.replace("https://user:passwd@github.com", "");
    generic_command_parse(uri_parse, &input, "/rust-lang/rust/issues");
}

#[test]
fn test_uri_to_path_fragments() {
    generic_parse(
        uri_to_path_fragments,
        "/rust-lang/rust/issues",
        vec!["rust-lang", "rust", "issues"],
    );
}

#[test]
fn test_queries_parse() {
    let input = TEST_URL_FULL.replace("https://user:passwd@github.com/rust-lang/rust/issues", "");
    generic_command_parse(queries_parse, &input, "?labels=E-easy&state=open");
}

#[test]
fn test_queries_to_query_fragments() {
    let expect = vec![
        ("labels".to_string(), "E-easy".to_string()),
        ("state".to_string(), "open".to_string()),
    ];
    generic_parse(
        queries_to_query_fragments,
        "?labels=E-easy&state=open",
        expect,
    );
}

#[test]
fn test_fragment_parse() {
    let input = TEST_URL_FULL.replace(
        "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open",
        "",
    );
    generic_command_parse(fragment_parse, &input, "ABC");
}
