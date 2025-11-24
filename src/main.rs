use clap::{Arg, ArgAction, Command};
use curl::{parse_curl_command, ParsedRequest};
use serde_json::{json, Value};

pub mod curl;
mod test_util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CurlCommand {
    Method,
    Header,
    Data,
    Flag,
    Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum JsonField {
    Url,
    Method,
    Headers,
    Data,
    Flags,
    Tokens,
}

fn main() {
    let matches = Command::new("nomcurl")
        .version("0.1.0")
        .about("A CLI tool to parse and manipulate curl commands")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("parse")
                .about("Parses a curl command")
                .arg(
                    Arg::new("command")
                        .help("The input curl command string")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("part")
                        .short('p')
                        .long("part")
                        .value_name("PART")
                        .help(
                            "Specifies which part of the curl command to parse (method, header, data, flag, url)",
                        )
                        .required(false)
                        .value_parser(clap::value_parser!(CurlCommand)),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .help("Outputs the parsed result as JSON")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("json-key")
                        .long("json-key")
                        .value_name("FIELD")
                        .help("Limits JSON output to specific fields (url, method, headers, data, flags, tokens)")
                        .value_parser(clap::value_parser!(JsonField))
                        .action(ArgAction::Append)
                        .requires("json"),
                )
                .arg(
                    Arg::new("pretty")
                        .long("pretty")
                        .help("Pretty-print JSON output (requires --json)")
                        .requires("json")
                        .action(ArgAction::SetTrue),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("parse", sub_matches)) => {
            let command = sub_matches.get_one::<String>("command").unwrap();
            let part = sub_matches.get_one::<CurlCommand>("part").copied();
            let output_json = sub_matches.get_flag("json");
            let pretty = sub_matches.get_flag("pretty");
            let json_keys: Vec<JsonField> = sub_matches
                .get_many::<JsonField>("json-key")
                .map(|vals| vals.copied().collect())
                .unwrap_or_default();

            match parse_curl_command(command) {
                Ok(parsed) => {
                    if output_json {
                        if let Err(err) = print_json_output(&parsed, part, pretty, &json_keys) {
                            print_json_error("serialization_error", &err.to_string(), pretty);
                        }
                    } else {
                        match part {
                            Some(part) => print_part(&parsed, part),
                            None => print_request_summary(&parsed),
                        }
                    }
                }
                Err(e) => {
                    if output_json {
                        print_json_error("parse_error", &e.to_string(), pretty);
                    } else {
                        eprintln!("Error parsing curl command: {e}");
                    }
                }
            }
        }
        _ => {
            Command::new("nomcurl").print_help().unwrap();
            println!();
        }
    }
}

fn print_part(parsed: &ParsedRequest, part: CurlCommand) {
    match part {
        CurlCommand::Method => match &parsed.method {
            Some(method) => println!("{method}"),
            None => println!("(method not specified)"),
        },
        CurlCommand::Header => {
            if parsed.headers.is_empty() {
                println!("(no headers)");
            } else {
                for header in &parsed.headers {
                    println!("{header}");
                }
            }
        }
        CurlCommand::Data => {
            if parsed.data.is_empty() {
                println!("(no data payload)");
            } else {
                for payload in &parsed.data {
                    println!("{payload}");
                }
            }
        }
        CurlCommand::Flag => {
            if parsed.flags.is_empty() {
                println!("(no flags)");
            } else {
                for flag in &parsed.flags {
                    println!("{flag}");
                }
            }
        }
        CurlCommand::Url => println!("{}", parsed.url),
    }
}

fn print_request_summary(parsed: &ParsedRequest) {
    println!("URL: {}", parsed.url);
    match &parsed.method {
        Some(method) => println!("Method: {method}"),
        None => println!("Method: (not specified)"),
    }

    if parsed.headers.is_empty() {
        println!("Headers: (none)");
    } else {
        println!("Headers:");
        for header in &parsed.headers {
            println!("  - {header}");
        }
    }

    if parsed.data.is_empty() {
        println!("Data: (none)");
    } else {
        println!("Data:");
        for payload in &parsed.data {
            println!("  - {payload}");
        }
    }

    if parsed.flags.is_empty() {
        println!("Flags: (none)");
    } else {
        println!("Flags:");
        for flag in &parsed.flags {
            println!("  - {flag}");
        }
    }
}

fn print_json_output(
    parsed: &ParsedRequest,
    part: Option<CurlCommand>,
    pretty: bool,
    keys: &[JsonField],
) -> Result<(), serde_json::Error> {
    let value = build_json_value(parsed, part, keys)?;
    let json_string = if pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    };

    println!("{}", json_string);
    Ok(())
}

fn build_json_value(
    parsed: &ParsedRequest,
    part: Option<CurlCommand>,
    keys: &[JsonField],
) -> Result<Value, serde_json::Error> {
    if let Some(part) = part {
        let value = match part {
            CurlCommand::Method => json!(parsed.method),
            CurlCommand::Header => json!(parsed.headers),
            CurlCommand::Data => json!(parsed.data),
            CurlCommand::Flag => json!(parsed.flags),
            CurlCommand::Url => json!(parsed.url),
        };
        return Ok(value);
    }

    if !keys.is_empty() {
        let mut map = serde_json::Map::new();
        for key in keys {
            match key {
                JsonField::Url => {
                    map.insert("url".into(), json!(parsed.url));
                }
                JsonField::Method => {
                    map.insert("method".into(), json!(parsed.method));
                }
                JsonField::Headers => {
                    map.insert("headers".into(), json!(parsed.headers));
                }
                JsonField::Data => {
                    map.insert("data".into(), json!(parsed.data));
                }
                JsonField::Flags => {
                    map.insert("flags".into(), json!(parsed.flags));
                }
                JsonField::Tokens => {
                    map.insert("tokens".into(), json!(parsed.tokens));
                }
            }
        }
        return Ok(Value::Object(map));
    }

    serde_json::to_value(parsed)
}

fn print_json_error(code: &str, message: &str, pretty: bool) {
    let payload = json!({
        "code": code,
        "error": message,
    });

    if pretty {
        if let Ok(output) = serde_json::to_string_pretty(&payload) {
            println!("{}", output);
        }
    } else {
        println!("{}", payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_json_value_with_keys() {
        let cmd = "curl 'https://example.com' -H 'A:1' --data name=value --insecure";
        let parsed = parse_curl_command(cmd).expect("parsed");
        let value = build_json_value(&parsed, None, &[JsonField::Url, JsonField::Headers])
            .expect("json value");
        assert!(value.get("url").is_some());
        assert!(value.get("headers").is_some());
        assert!(value.get("data").is_none());
    }

    #[test]
    fn test_build_json_value_part_overrides_keys() {
        let cmd = "curl 'https://example.com' --data name=value";
        let parsed = parse_curl_command(cmd).expect("parsed");
        let value = build_json_value(&parsed, Some(CurlCommand::Data), &[JsonField::Url])
            .expect("json value");
        assert!(value.is_array());
    }
}
