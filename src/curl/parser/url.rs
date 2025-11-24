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
    let queries = input.strip_prefix('?').unwrap_or(input);

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
