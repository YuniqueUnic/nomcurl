use nom::{
    bytes::complete::{tag, take_till},
    character::complete::{alpha1, alphanumeric0, alphanumeric1, multispace0},
    combinator::{map, opt},
    error::{context, Error, ErrorKind},
    sequence::preceded,
    IResult, Parser,
};

use crate::curl::url::{CurlUrl, UserInfo};

pub fn curl_url_parse(input: &str) -> IResult<&str, CurlUrl> {
    context(
        "curl_url_parse",
        (
            protocol_parse,
            credentials_domain_parse,
            opt(uri_parse),
            opt(queries_parse),
            opt(fragment_parse),
        )
            .map_res(|(protocol, credentials, uri, queries, fragment)| {
                let (_, host) = credentials_domain_to_host_parse(credentials)?;
                let mut curl_url = CurlUrl::new(&protocol, host);

                if let Some(uri) = uri {
                    curl_url.set_uri(uri);
                }

                if let Some(queries) = queries {
                    let fragments = queries_to_query_fragments(queries);
                    curl_url.set_queries(fragments);
                }

                if let Some(fragment) = fragment {
                    curl_url.set_fragment(fragment);
                }

                if let Ok((_, userinfo)) = credentials_domain_to_userinfo_parse(credentials) {
                    if let Some(ui) = UserInfo::from_raw(userinfo) {
                        curl_url.set_userinfo(ui);
                    }
                }

                Ok::<_, nom::Err<Error<&str>>>(curl_url)
            }),
    )
    .parse(input)
}

pub fn protocol_parse(input: &str) -> IResult<&str, String> {
    context(
        "protocol_parse",
        preceded(
            multispace0,
            map(
                (
                    alpha1,
                    alphanumeric0,
                    nom::character::complete::char(':'),
                    tag("//"),
                ),
                |(p1, p2, _, _)| format!("{}{}", p1, p2),
            ),
        ),
    )
    .parse(input)
}

pub fn credentials_domain_parse(input: &str) -> IResult<&str, &str> {
    context("credentials_domain_parse", take_till(|c| c == '/')).parse(input)
}

pub fn credentials_domain_to_userinfo_parse(input: &str) -> IResult<&str, &str> {
    if let Some(at_index) = input.find('@') {
        let userinfo = &input[..at_index];
        Ok((&input[at_index + 1..], userinfo))
    } else {
        Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail)))
    }
}

pub fn credentials_domain_to_host_parse(input: &str) -> IResult<&str, &str> {
    if let Some(at_index) = input.find('@') {
        Ok((&input[..at_index], &input[at_index + 1..]))
    } else {
        Ok(("", input))
    }
}

pub fn uri_parse(input: &str) -> IResult<&str, &str> {
    context("uri_parse", take_till(|c| c == '?')).parse(input)
}

pub fn uri_to_path_fragments(input: &str) -> Vec<&str> {
    input.split('/').filter(|pf| !pf.is_empty()).collect()
}

pub fn queries_parse(input: &str) -> IResult<&str, &str> {
    context("queries_parse", take_till(|c| c == '#')).parse(input)
}

pub fn queries_to_query_fragments(input: &str) -> Vec<(String, String)> {
    let queries = if input.starts_with('?') {
        &input[1..]
    } else {
        input
    };

    queries
        .split('&')
        .filter(|fragment| !fragment.is_empty())
        .map(|query| {
            let mut parts = query.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            (key.into(), value.into())
        })
        .collect()
}

pub fn fragment_parse(input: &str) -> IResult<&str, &str> {
    context(
        "fragment_parse",
        map(
            (nom::character::complete::char('#'), alphanumeric1),
            |(_, fragment)| fragment,
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{generic_command_parse, generic_parse};

    const TEST_URL_FULL: &str =
        "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open#ABC";

    #[test]
    fn test_curl_url_parse() {
        let input = TEST_URL_FULL;
        let userinfo = UserInfo::from_raw("user:passwd").unwrap();
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
        let expect = "https";

        generic_command_parse(protocol_parse, input, expect.into());

        let expect = "smb";
        let input = input.replace("https", expect);

        generic_command_parse(protocol_parse, &input, expect.into());

        let expect = "FTP";
        let input = input.replace("smb", expect);

        generic_command_parse(protocol_parse, &input, expect.into());
    }

    #[test]
    fn test_credentials_domain_parse() {
        let input = TEST_URL_FULL.replace("https://", "");
        let expect = "user:passwd@github.com";

        generic_command_parse(credentials_domain_parse, &input, expect);
    }

    #[test]
    fn test_credentials_domain_to_userinfo_parse() {
        let input = "user:passwd@github.com";
        let expect = "user:passwd";

        generic_command_parse(credentials_domain_to_userinfo_parse, input, expect);
    }

    #[test]
    fn test_credentials_domain_to_host_parse() {
        let input = "user:passwd@github.com";
        let expect = "github.com";

        let (_, host) = credentials_domain_to_host_parse(input).expect("host");
        assert_eq!(host, expect);
    }

    #[test]
    fn test_uri_parse() {
        let input = TEST_URL_FULL.replace("https://user:passwd@github.com", "");
        let expect = "/rust-lang/rust/issues";

        generic_command_parse(uri_parse, &input, expect);
    }

    #[test]
    fn test_uri_to_path_fragments() {
        let input = "/rust-lang/rust/issues";
        let expect = vec!["rust-lang", "rust", "issues"];

        generic_parse(uri_to_path_fragments, input, expect);
    }

    #[test]
    fn test_queries_parse() {
        let input =
            TEST_URL_FULL.replace("https://user:passwd@github.com/rust-lang/rust/issues", "");
        let expect = "?labels=E-easy&state=open";

        generic_command_parse(queries_parse, &input, expect);
    }

    #[test]
    fn test_queries_to_query_fragments() {
        let input = "?labels=E-easy&state=open";
        let expect = vec![
            ("labels".to_string(), "E-easy".to_string()),
            ("state".to_string(), "open".to_string()),
        ];

        generic_parse(queries_to_query_fragments, input, expect);
    }

    #[test]
    fn test_fragment_parse() {
        let input = TEST_URL_FULL.replace(
            "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open",
            "",
        );
        let expect = "ABC";

        generic_command_parse(fragment_parse, &input, expect);
    }
}
