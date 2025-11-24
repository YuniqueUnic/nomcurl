# nomcurl: Parse cURL Commands with Nom

[中文](./readme-cn.md) | [English](./readme.md)

`nomcurl` is a Rust library and CLI dedicated to turning raw `curl …` strings into structured data. It now targets `nom` 8, exposes a stable `parse_curl_command` API, and keeps the CLI focused on fast inspection of individual parts of a request.

## Highlights

- **Typed parser output**: `parse_curl_command` returns a `ParsedRequest` struct with URL, method, headers, payloads, and flags.
- **Nom 8 ready**: Internal parsers rely on the modern `Parser` trait instead of deprecated helpers.
- **Modular codebase**: The crate is split into `curl::command`, `curl::parser`, `curl::url`, and `curl::request` for easier maintenance and extension.
- **Drop-in CLI**: `cargo run -- parse "…"` lets you print the whole request or specific slices (`--part header`, `--part flag`, etc.).
- **Programmable outputs**: Consume raw tokens via `ParsedRequest::tokens` or ask the CLI for JSON using `--json` / `--pretty`.
- **Flexible payload parsing**: Handles `--form`, `--form-string`, `--data-binary @file`, and other unquoted values alongside the traditional quoted style.
- **Richer flag support**: Long options like `--compressed`, `--retry 3`, `--cookie-jar` are parsed consistently, ensuring they appear in `ParsedRequest::flags`.

## Installation

```toml
[dependencies]
nomcurl = "0.2.0"
```

## Library Usage

```rust
use nomcurl::{parse_curl_command, ParsedRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = "curl 'https://api.example.com' -X POST \
        -H 'Accept: application/json' -d 'name=Ann' --insecure";

let parsed: ParsedRequest = parse_curl_command(command)?;
assert_eq!(parsed.method.as_deref(), Some("POST"));
assert_eq!(parsed.flags, vec!["--insecure".to_string()]);
assert_eq!(parsed.tokens.len(), 4); // URL + header + data + flag

println!("URL -> {}", parsed.url);
println!("Headers -> {:?}", parsed.headers);
Ok(())
}
```

If you need lower-level access, the root module re-exports the building blocks:

```rust
use nomcurl::{curl_cmd_parse, Curl};

let (_, tokens): (_, Vec<Curl>) = curl_cmd_parse(command).expect("valid curl");
```

## CLI

```bash
# Print an overview
cargo run -- parse "curl 'https://httpbin.org/get' -H 'Accept: */*'"

# Extract only the headers
cargo run -- parse "curl 'https://httpbin.org/post' -H 'A:1' -H 'B:2'" --part header

# Emit JSON (pretty printed) and limit to select keys
cargo run -- parse "curl 'https://httpbin.org/post' --data name=value --insecure" \
  --json --pretty --json-key url --json-key data
```

The CLI mirrors the library parser, so every fix automatically benefits both surfaces.

## Module Layout

- `curl::command`: Core data structures (`CurlField`, `CurlToken`) and macros for tests.
- `curl::parser`: Nom parsers split into `common`, `command`, and `url` submodules.
- `curl::url`: Immutable `CurlUrl`, `Protocol`, and `UserInfo` types.
- `curl::request`: High-level `ParsedRequest` plus `parse_curl_command` and error types.

## Testing

```bash
cargo test
```

All unit tests cover both the parser internals and the public CLI.

## License

MIT
