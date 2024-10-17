use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_until, take_while},
    character::{
        self,
        complete::{char, line_ending, multispace0, multispace1, none_of, one_of, space1},
        streaming::alphanumeric0,
    },
    combinator::{all_consuming, eof, map, recognize, rest},
    error::{context, Error, ErrorKind, ParseError},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use super::Curl;

const CURL_CMD: &str = "curl";
pub fn is_curl(input: &str) -> bool {
    input.trim_start().to_lowercase().starts_with(CURL_CMD)
}

pub fn remove_curl_cmd_header(input: &str) -> &str {
    &input[4..]
}

/// Identify the ending pattern: <space+>\<space*>\r\n
pub fn slash_line_ending(input: &str) -> IResult<&str, &str> {
    context(
        "Slash line ending",
        recognize(tuple((
            multispace1,
            character::complete::char('\\'),
            multispace0,
            line_ending,
        ))),
    )(input)
}

/// Parse double-quoted data with support for escaped characters
fn double_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "Double quoted data parse",
        preceded(
            tuple((multispace0, char('\"'))),
            terminated(take_until("\""), tuple((char('\"'), multispace0))),
        ),
    )(input)
}

/// Parse single-quoted data with support for escaped characters
fn single_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "Single quoted data parse",
        preceded(
            tuple((multispace0, char('\''))),
            terminated(take_until("\'"), tuple((char('\''), multispace0))),
        ),
    )(input)
}

/// Get the longest one quoted data between single / double quoted data.
fn quoted_data_parse<'a>(input: &str) -> IResult<&str, &str> {
    let double_res = double_quoted_data_parse(input);
    let single_res = single_quoted_data_parse(input);

    if double_res.is_ok() && single_res.is_ok() {
        let (_double_rest, double_data) = double_res.unwrap();
        let (_single_rest, single_data) = single_res.unwrap();

        if double_data.len() >= single_data.len() {
            Ok((_double_rest, double_data))
        } else {
            Ok((_single_rest, single_data))
        }
    } else if double_res.is_err() && single_res.is_ok() {
        single_res
    } else if double_res.is_ok() && single_res.is_err() {
        double_res
    } else {
        // Both parsing failed, return an error
        let single_err = single_res.unwrap_err();
        let double_err = double_res.unwrap_err();
        eprintln!(
            "The origin: ({})\r\nThe single parse error: {}\r\nThe double parse error: {}",
            input, single_err, double_err
        );
        Err(nom::Err::Failure(Error::new(&input, ErrorKind::Fail)))
    }
}

pub fn iter_quoted_data_parse(input: &str) -> IResult<&str, Vec<String>> {
    context(
        "Iter quoted data parse",
        fold_many0(
            alt((double_quoted_data_parse, single_quoted_data_parse)),
            Vec::new,
            |mut acc: Vec<String>, item| {
                acc.push(item.into());
                acc
            },
        ),
    )(input)
}

pub fn method_parse(input: &str) -> IResult<&str, Curl> {
    context(
        "Method parse",
        preceded(
            multispace0,
            terminated(
                map(
                    tuple((tag("-X"), multispace1, quoted_data_parse)),
                    |(method, _space, data)| Curl::new(method, data).unwrap(),
                ),
                alt((space1, eof, slash_line_ending, line_ending)),
            ),
        ),
    )(input)
}

pub fn url_parse(input: &str) -> IResult<&str, Curl> {
    todo!()
}
pub fn headers_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    todo!()
}

pub fn flags_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use nom::InputTake;

    use super::*;

    trait StrExtensions {
        fn exchange_quotes(&self) -> String;
    }

    impl StrExtensions for str {
        fn exchange_quotes(&self) -> String {
            let mut result = String::with_capacity(self.len());

            for c in self.chars() {
                match c {
                    '"' => result.push('\''),
                    '\'' => result.push('\"'),
                    _ => result.push(c),
                }
            }

            result
        }
    }

    #[test]
    fn test_is_curl() {
        let cmd = "\t \r  \n Curl asdjfnv\n";
        assert!(is_curl(&cmd));
        let cmd = cmd.trim().to_uppercase();
        assert!(is_curl(&cmd));
    }

    #[test]
    fn test_remove_curl_cmd_headr() {
        let cmd = "\t \r  \n Curl asdjfnv\n".trim_start();
        let len = &cmd.len();
        let cmd = remove_curl_cmd_header(cmd);
        assert_eq!(len - 4, cmd.len(), "current cmd is: ({})", cmd);
        assert_ne!("l", cmd.take(1));
        assert_eq!(" ", cmd.take(1))
    }

    #[test]
    fn test_single_quoted_data_parse() {
        let expect = " hhdf,\\fjsdfjl**''";
        let input = format!(
            r##"{}{}"{}" woaini "{}'nmihao'"##,
            "\t \r  \n ", "\n ", expect, " \r \n "
        )
        .exchange_quotes();

        let result = single_quoted_data_parse(&input);
        assert!(result.is_ok(), "The result is: ({:?})", result);

        let (_rest, data) = result.unwrap();
        assert_eq!(
            expect.exchange_quotes(),
            data,
            "The expect:\r\n({}) should be same with the data:\r\n({})",
            expect,
            data
        );
    }

    #[test]
    fn test_double_quoted_data_parse() {
        let expect = r#" hhdf,\\fjsdfjl**''"#;
        let input = format!(
            r##"{}{}"{}" woaini "{}'nmihao'"##,
            "\t \r  \n ", "\n ", expect, " \r \n "
        );
        // println!("{}", input);
        let result = double_quoted_data_parse(&input);
        // println!("{:?}", result);
        assert!(result.is_ok());
        let (_rest, data) = result.unwrap();
        assert_eq!(
            expect, data,
            "The expect:\r\n({}) should be same with the data:\r\n({})",
            expect, data
        );
    }

    #[test]
    fn test_quoted_data_parse() {
        let expect = " hhdf,\\fjsdfjl**''";
        let input = format!("\t \r  \n \n \"{}\" woaini \" \r \n 'nmihao'", expect);
        let result = quoted_data_parse(&input);
        assert!(!result.is_err());
        let (_rest, data) = result.unwrap();
        assert_eq!(
            expect, data,
            "The expect:\r\n({}) should be same with the data:\r\n({})",
            expect, data
        );
    }

    #[test]
    fn test_iter_quoted_data_parse() {
        let expect = vec![" hhdf,\\fjsdfjl**''", "nmihao"];
        let input = format!("\t \r  \n \n \"{}\"   \r \n '{}'", expect[0], expect[1]);
        let result = iter_quoted_data_parse(&input);
        assert!(!result.is_err());
        let (_rest, data) = result.unwrap();
        assert_eq!(
            expect, data,
            "The expect:\r\n({:#?}) should be same with the data:\r\n({:#?})",
            expect, data
        );

        let expect = vec![" hhdf,\\fjsdfjl**''", "nmihao"];
        let input = format!("\t \r  \n \n \"{}\" \r \n \"{}\"", expect[0], expect[1]);
        let result = iter_quoted_data_parse(&input);
        assert!(!result.is_err());
        let (_rest, data) = result.unwrap();
        assert_eq!(
            expect, data,
            "The expect:\r\n({:#?}) should be same with the data:\r\n({:#?})",
            expect, data
        );
    }

    #[test]
    fn test_method_parse() {
        let cmd = "\t \r  \n Curl asdjfnv\n".trim_start();
        let len = &cmd.len();
        let cmd = remove_curl_cmd_header(cmd);
        assert_eq!(len - 4, cmd.len());
        assert_ne!("l", cmd.take(1));
        assert_eq!(" ", cmd.take(1))
    }

    #[test]
    fn test_url_parse() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_headers_parse() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_flags_parse() {
        assert_eq!(1, 1);
    }
}
