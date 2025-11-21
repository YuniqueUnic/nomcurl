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

use super::common::{is_curl, quoted_data_parse, remove_curl_cmd_header, slash_line_ending};
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
                    quoted_data_parse,
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
    "-d",
    "--data",
    "--data-raw",
    "--data-binary",
    "--data-urlencode"
);
parse_commands!(datas_parse, data_parse);
parse_commands!(flags_parse, flag_parse);

pub fn flag_parse(input: &str) -> IResult<&str, Curl> {
    context(
        "flag parse",
        (
            opt(slash_line_ending),
            multispace0,
            recognize((
                character::complete::char('-'),
                opt(character::complete::char('-')),
                take_while1(|c: char| c.is_alphanumeric() || matches!(c, '-' | '_')),
            )),
        )
            .map_res(|(_, _, flag)| {
                if Curl::expects_value(flag) {
                    return Err(nom::Err::Failure::<Error<&str>>(Error::new(
                        flag,
                        ErrorKind::Fail,
                    )));
                }
                Curl::new_flag(flag).ok_or_else(|| {
                    nom::Err::Failure::<Error<&str>>(Error::new(flag, ErrorKind::Fail))
                })
            }),
    )
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

#[cfg(test)]
mod tests {
    use crate::curl;
    use crate::curl::parser::common;
    use crate::new_curl;
    use crate::test_util::generic_command_parse;

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
                    '\'' => result.push('"'),
                    _ => result.push(c),
                }
            }

            result
        }
    }

    const TEST_CURL_CMD_FULL: &str = r#"
        curl 'http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498'  \
        -H 'Accept: */*' -X 'TEST' \
        -H 'Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7' --b \
        -H 'Cache-Control: no-cache' \
        -H 'Connection: keep-alive' \
        -d 'data1:90' \
        --data 'data2:90/i9fi0sdfsdfk\\jfhaoe' \
        -H 'Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}' \
        -H 'Pragma: no-cache' \
        -H 'Referer: http://www.sse.com.cn/'  \
        -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36' \
        --insecure
    "#;

    #[test]
    fn test_curl_cmd_parse() {
        let full_url_str = "http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498";

        let (_, expect_url) = match curl::parser::curl_url_parse(full_url_str) {
            Ok(u) => u,
            Err(e) => panic!("Error: {:?}", e),
        };

        let url_token = Curl::new_url(expect_url);

        let expect = vec![
            url_token,
            new_curl!(-H, "Accept: */*"),
            new_curl!(-X, "TEST"),
            new_curl!(-H, "Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
            new_curl!("--b"),
            new_curl!(-H, "Cache-Control: no-cache"),
            new_curl!(-H, "Connection: keep-alive"),
            new_curl!(-d, "data1:90"),
            new_curl!(-d, "data2:90/i9fi0sdfsdfk\\\\jfhaoe"),
            new_curl!(
                -H,
                "Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}"
            ),
            new_curl!(-H, "Pragma: no-cache"),
            new_curl!(-H, "Referer: http://www.sse.com.cn/"),
            new_curl!(
                -H,
                "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
            ),
            new_curl!("--insecure"),
        ];

        generic_command_parse(curl_cmd_parse, TEST_CURL_CMD_FULL, expect);
    }

    #[test]
    fn test_is_curl() {
        let cmd = "\t \r  \n Curl asdjfnv\n";
        assert!(is_curl(cmd));
        let cmd = cmd.trim().to_uppercase();
        assert!(is_curl(&cmd));
    }

    #[test]
    fn test_remove_curl_cmd_header() {
        let cmd = "\t \r  \n Curl asdjfnv\n";
        let trimmed = cmd.trim_start();
        let stripped = remove_curl_cmd_header(trimmed);
        assert!(!stripped.starts_with('c'));
        assert!(stripped.starts_with(' '));
    }

    #[test]
    fn test_url_parse() {
        let full_url_str = "http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498";
        let (_, expect_url) = match curl::parser::curl_url_parse(full_url_str) {
            Ok(u) => u,
            Err(e) => panic!("Error: {:?}", e),
        };

        let expect = Curl::new_url(expect_url);
        let input = format!(" curl \r \t   '{}' \\ \r\n-H 'Accept: */*'", full_url_str);
        let input = remove_curl_cmd_header(&input.trim_start());

        generic_command_parse(url_parse, input, expect);
    }

    #[test]
    fn test_commands_parse() {
        let expect = vec![
            new_curl!(-H, "Accept: */*"),
            new_curl!(-X, "TEST"),
            new_curl!(-H, "Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
            new_curl!("--b"),
            new_curl!(-H, "Cache-Control: no-cache"),
            new_curl!(-H, "Connection: keep-alive"),
            new_curl!(-d, "data1:90"),
            new_curl!(-d, "data2:90/i9fi0sdfsdfk\\\\jfhaoe"),
            new_curl!(
                -H,
                "Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}"
            ),
            new_curl!(-H, "Pragma: no-cache"),
            new_curl!(-H, "Referer: http://www.sse.com.cn/"),
            new_curl!(
                -H,
                "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
            ),
            new_curl!("--insecure"),
        ];

        let input = r#"
         \
        -H 'Accept: */*' -X 'TEST' \
        -H 'Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7' --b \
        -H 'Cache-Control: no-cache' \
        -H 'Connection: keep-alive' \
        -d 'data1:90' \
        --data 'data2:90/i9fi0sdfsdfk\\jfhaoe' \
        -H 'Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}' \
        -H 'Pragma: no-cache' \
        -H 'Referer: http://www.sse.com.cn/'  \
        -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36' \
        --insecure
        "#;

        generic_command_parse(commands_parse, input, expect);
    }

    #[test]
    fn test_single_quoted_data_parse() {
        let expect = " hhdf,\\fjsdfjl**''";
        let input = format!(
            r##"{}{}"{}" woaini "{}'nmihao'"##,
            "\t \r  \n ", "\n ", expect, " \r \n "
        )
        .exchange_quotes();

        generic_command_parse(
            common::single_quoted_data_parse,
            &input,
            &expect.exchange_quotes(),
        );
    }

    #[test]
    fn test_double_quoted_data_parse() {
        let expect = r#" hhdf,\\fjsdfjl**''"#;
        let input = format!(
            r##"{}{}"{}" woaini "{}'nmihao'"##,
            "\t \r  \n ", "\n ", expect, " \r \n "
        );

        generic_command_parse(common::double_quoted_data_parse, &input, expect);
    }

    #[test]
    fn test_quoted_data_parse() {
        let expect = " hhdf,\\fjsdfjl**''";
        let input = format!("\t \r  \n \n \"{}\" woaini \" \r \n 'nmihao'", expect);
        generic_command_parse(common::quoted_data_parse, &input, expect);
    }

    #[test]
    fn test_iter_quoted_data_parse() {
        let expect: Vec<String> = vec![" hhdf,\\fjsdfjl**''".into(), "nmihao".into()];
        let input = format!("\t \r  \n \n \"{}\"   \r \n '{}'", expect[0], expect[1]);

        generic_command_parse(common::iter_quoted_data_parse, &input, expect.clone());

        let input = format!("\t \r  \n \n \"{}\" \r \n \"{}\"", expect[0], expect[1]);

        generic_command_parse(common::iter_quoted_data_parse, &input, expect);
    }

    #[test]
    fn test_data_parse() {
        let expect = new_curl!(-d, "AJFjfdslf");
        let input = "\t \r  \n -d \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(data_parse, input, expect);
    }

    #[test]
    fn test_datas_parse() {
        let expect = vec![
            new_curl!(-d, "AJFjfdslf"),
            new_curl!(-d, "abc fjdfl  ii\\hhfjsdkf:90"),
        ];
        let input = "\t \r  \n -d \"AJFjfdslf\" --data \"abc fjdfl  ii\\hhfjsdkf:90\" \t\r jflksfl";
        generic_command_parse(datas_parse, input, expect);
    }

    #[test]
    fn test_method_parse() {
        let expect = new_curl!(-X, "AJFjfdslf");
        let input = "\t \r  \n -X \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(method_parse, input, expect);
    }

    #[test]
    fn test_methods_parse() {
        let expect = vec![
            new_curl!(-X, "AJFjfdslf"),
            new_curl!(-X, "abc fjdfl  ii\\hhfjsdkf:90"),
        ];
        let input = "\t \r  \n -X \"AJFjfdslf\" -X \"abc fjdfl  ii\\hhfjsdkf:90\" \t\r jflksfl";
        generic_command_parse(methods_parse, input, expect);
    }

    #[test]
    fn test_header_parse() {
        let expect = new_curl!(-H, "AJFjfdslf");
        let input = "\t \r  \n -H \"AJFjfdslf\" HHH -H \"llol:90\" -X a";
        generic_command_parse(header_parse, input, expect);
    }

    #[test]
    fn test_headers_parse() {
        let expect = vec![
            new_curl!(-H, "AJFjfdslf"),
            new_curl!(-H, "abc fjdfl  ii\\hhfjsdkf:90"),
        ];
        let input = "\t \r  \n -H \"AJFjfdslf\" -H \"abc fjdfl  ii\\hhfjsdkf:90\" \t\r jflksfl";
        generic_command_parse(headers_parse, input, expect);
    }

    #[test]
    fn test_flag_parse() {
        let expect = new_curl!("--help");
        let input = "\t \r --help -a  \n -X \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(flag_parse, input, expect);
    }

    #[test]
    fn test_flags_parse() {
        let expect = vec![new_curl!("--help"), new_curl!("-a")];
        let input = "\t \r --help -a  \n -X \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(flags_parse, input, expect);
    }
}
