use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::{
        self,
        complete::{multispace0, multispace1},
    },
    combinator::{opt, recognize},
    error::{context, Error, ErrorKind},
    multi::fold_many0,
    IResult, Parser,
};

use crate::curl::Curl;

use super::common::{
    argument_value_parse, is_curl, quoted_data_parse, remove_curl_cmd_header, slash_line_ending,
};
use super::url::curl_url_parse;

pub fn url_parse(input: &str) -> IResult<&str, Curl> {
    context(
        "url parse",
        (multispace0, quoted_data_parse)
            .map_res(|(_, data)| curl_url_parse(data).map(|(_, parsed)| Curl::new_url(parsed))),
    )
    .parse(input)
}

macro_rules! parse_command {
    ($name:ident,$($tag:expr),+) => {
        pub fn $name(input: &str) -> IResult<&str, Curl> {
            context(
                stringify!($name),
                (
                    opt(slash_line_ending),
                    multispace0,
                    alt(($(tag($tag)),+,)),
                    multispace1,
                    argument_value_parse,
                )
                    .map_res(|(_, _, method, _, data)| {
                        Curl::new(method, data).ok_or_else(|| {
                            nom::Err::Failure::<Error<&str>>(Error::new(data, ErrorKind::Fail))
                        })
                    }),
            )
            .parse(input)
        }
    };
}

macro_rules! parse_commands {
    ($name:ident,$inner_func:ident) => {
        pub fn $name(input: &str) -> IResult<&str, Vec<Curl>> {
            context(
                stringify!($name),
                fold_many0($inner_func, Vec::new, |mut acc: Vec<Curl>, item| {
                    acc.push(item);
                    acc
                }),
            )
            .parse(input)
        }
    };
}

parse_command!(method_parse, "-X", "--request");
parse_commands!(methods_parse, method_parse);
parse_command!(header_parse, "-H", "--header");
parse_commands!(headers_parse, header_parse);
parse_command!(
    data_parse,
    "--data-urlencode",
    "--data-binary",
    "--data-raw",
    "--data",
    "-d",
    "--form-string",
    "--form",
    "-F"
);
parse_commands!(datas_parse, data_parse);
parse_commands!(flags_parse, flag_parse);

pub fn flag_parse(input: &str) -> IResult<&str, Curl> {
    context("flag parse", |input| {
        let (input, _) = opt(slash_line_ending).parse(input)?;
        let (input, _) = multispace0(input)?;
        let (mut input, flag) = recognize((
            character::complete::char('-'),
            opt(character::complete::char('-')),
            take_while1(|c: char| c.is_alphanumeric() || matches!(c, '-' | '_')),
        ))
        .parse(input)?;

        if Curl::expects_value(flag) {
            return Err(nom::Err::Error(Error::new(flag, ErrorKind::Fail)));
        }

        let mut value: Option<&str> = None;
        if Curl::flag_requires_value(flag) {
            let (after_space, _) = multispace1(input)?;
            if let Some(next_char) = after_space.chars().next() {
                if next_char == '-' {
                    return Err(nom::Err::Error(Error::new(after_space, ErrorKind::Fail)));
                }
            }
            let (after_value, parsed_value) = argument_value_parse(after_space)?;
            value = Some(parsed_value);
            input = after_value;
        }

        let curl = Curl::new_flag_with_value(flag, value)
            .ok_or_else(|| nom::Err::Error(Error::new(flag, ErrorKind::Fail)))?;

        Ok((input, curl))
    })
    .parse(input)
}

pub fn commands_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    context(
        "all commands parse",
        fold_many0(
            alt((method_parse, header_parse, data_parse, flag_parse)),
            Vec::new,
            |mut acc, item| {
                acc.push(item);
                acc
            },
        ),
    )
    .parse(input)
}

pub fn curl_cmd_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    if !is_curl(input) {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Fail)));
    }

    let trimmed = remove_curl_cmd_header(input);
    let (rest_after_url, url_token) = url_parse(trimmed)?;

    let mut curl_cmds = vec![url_token];
    let (rest, mut additional) = context("curl cmd parse", commands_parse).parse(rest_after_url)?;
    curl_cmds.append(&mut additional);
    Ok((rest, curl_cmds))
}
