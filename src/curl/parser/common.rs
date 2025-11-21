use nom::character::complete::multispace0;
use nom::{
    branch::alt,
    bytes::complete::{take_till1, take_until},
    character::{self, complete::char},
    combinator::recognize,
    error::{context, Error, ErrorKind},
    multi::fold_many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};

pub const CURL_CMD: &str = "curl";

pub fn is_curl(input: &str) -> bool {
    input
        .trim_start()
        .to_ascii_lowercase()
        .starts_with(CURL_CMD)
}

pub fn remove_curl_cmd_header(input: &str) -> &str {
    let trimmed = input.trim_start();
    if trimmed.len() >= CURL_CMD.len() && trimmed[..CURL_CMD.len()].eq_ignore_ascii_case(CURL_CMD) {
        &trimmed[CURL_CMD.len()..]
    } else {
        trimmed
    }
}

pub fn slash_line_ending(input: &str) -> IResult<&str, &str> {
    context(
        "slash line ending",
        recognize((multispace0, character::complete::char('\\'), multispace0)),
    )
    .parse(input)
}

pub fn double_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "double quoted data parse",
        delimited(
            (multispace0, char('"')),
            take_until("\""),
            (char('"'), multispace0),
        ),
    )
    .parse(input)
}

pub fn single_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "single quoted data parse",
        delimited(
            (multispace0, char('\'')),
            take_until("'"),
            (char('\''), multispace0),
        ),
    )
    .parse(input)
}

pub fn unquoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "unquoted data parse",
        preceded(
            multispace0,
            take_till1(|c: char| c.is_whitespace() || c == '\\'),
        ),
    )
    .parse(input)
}

pub fn quoted_data_parse(input: &str) -> IResult<&str, &str> {
    let double_res = double_quoted_data_parse(input);
    let single_res = single_quoted_data_parse(input);

    match (double_res, single_res) {
        (Ok(double_ok), Ok(single_ok)) => {
            if double_ok.1.len() >= single_ok.1.len() {
                Ok(double_ok)
            } else {
                Ok(single_ok)
            }
        }
        (Err(_), Ok(single_ok)) => Ok(single_ok),
        (Ok(double_ok), Err(_)) => Ok(double_ok),
        (Err(_single_err), Err(_double_err)) => {
            #[cfg(feature = "debug-print")]
            eprintln!(
                "The origin: ({})\r\nThe single parse error: {}\r\nThe double parse error: {}",
                input, _single_err, _double_err
            );

            Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail)))
        }
    }
}

pub fn argument_value_parse(input: &str) -> IResult<&str, &str> {
    context(
        "argument value parse",
        alt((
            double_quoted_data_parse,
            single_quoted_data_parse,
            unquoted_data_parse,
        )),
    )
    .parse(input)
}

pub fn iter_quoted_data_parse(input: &str) -> IResult<&str, Vec<String>> {
    context(
        "iter quoted data parse",
        fold_many0(
            alt((double_quoted_data_parse, single_quoted_data_parse)),
            Vec::new,
            |mut acc: Vec<String>, item| {
                acc.push(item.into());
                acc
            },
        ),
    )
    .parse(input)
}
