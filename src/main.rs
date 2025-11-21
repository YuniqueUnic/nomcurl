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

            match parse_curl_command(command) {
                Ok(parsed) => {
                    if output_json {
                        if let Err(err) = print_json_output(&parsed, part, pretty) {
                            eprintln!("Failed to serialize parsed request: {err}");
                        }
                    } else {
                        match part {
                            Some(part) => print_part(&parsed, part),
                            None => print_request_summary(&parsed),
                        }
                    }
                }
                Err(e) => eprintln!("Error parsing curl command: {e}"),
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
) -> Result<(), serde_json::Error> {
    let value: Value = match part {
        Some(CurlCommand::Method) => json!(parsed.method),
        Some(CurlCommand::Header) => json!(parsed.headers),
        Some(CurlCommand::Data) => json!(parsed.data),
        Some(CurlCommand::Flag) => json!(parsed.flags),
        Some(CurlCommand::Url) => json!(parsed.url),
        None => serde_json::to_value(parsed)?,
    };

    let json_string = if pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    };

    println!("{}", json_string);
    Ok(())
}
