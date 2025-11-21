# nomcurl：基于 Nom 的 cURL 命令解析器

[中文](./readme-cn.md) | [English](./readme.md)

`nomcurl` 是一个用于解析 `curl …` 命令的 Rust 库与 CLI。项目已经迁移到 `nom` 8，提供结构化的 `ParsedRequest` API，并对代码结构、性能与 CLI 体验进行了全面重构。

## 亮点

- **结构化输出**：`parse_curl_command` 返回 `ParsedRequest`，包含 URL、Method、Headers、Data 与 Flags。
- **兼容 nom 8**：内部解析器改用新的 `Parser` trait，实现零 copy 的组合式解析。
- **模块化代码**：代码拆分为 `curl::command`、`curl::parser`、`curl::url`、`curl::request` 等子模块，方便扩展与维护。
- **CLI 同步升级**：`cargo run -- parse "…"` 即可查看完整解析结果，或通过 `--part` 仅查看某部分。
- **编程友好**：`ParsedRequest::tokens` 提供原始 token 列表，CLI 也支持 `--json` / `--pretty` 输出。
- **更灵活的数据解析**：新增 `--form`/`--form-string`/`--data-binary @file` 以及未加引号 payload 的解析能力。

## 安装

```toml
[dependencies]
nomcurl = "0.2.0"
```

## 库使用示例

```rust
use nomcurl::{parse_curl_command, ParsedRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = "curl 'https://api.example.com' -X POST \
        -H 'Accept: application/json' -d 'name=Ann' --insecure";

let parsed: ParsedRequest = parse_curl_command(command)?;
assert_eq!(parsed.method.as_deref(), Some("POST"));
println!("URL -> {}", parsed.url);
println!("Headers -> {:?}", parsed.headers);
println!("Tokens -> {}", parsed.tokens.len());
Ok(())
}
```

若需要访问底层 token，可直接使用 `curl_cmd_parse` 与 `Curl`：

```rust
use nomcurl::{curl_cmd_parse, Curl};

let (_, tokens): (_, Vec<Curl>) = curl_cmd_parse(command).expect("valid curl");
```

## CLI

```bash
cargo run -- parse "curl 'https://httpbin.org/get' -H 'Accept: */*'"
cargo run -- parse "curl 'https://httpbin.org/post' -H 'A:1' -H 'B:2'" --part header
cargo run -- parse "curl 'https://httpbin.org/post' --data name=value --insecure" --json --pretty
```

## 模块总览

- `curl::command`：`CurlField`、`CurlToken` 以及测试宏。
- `curl::parser`：公共工具、命令解析、URL 解析等子模块。
- `curl::url`：`CurlUrl`、`Protocol`、`UserInfo` 等不可变结构体。
- `curl::request`：`ParsedRequest`、`parse_curl_command` 以及错误类型。

## 测试

```bash
cargo test
```

全部单元测试会同时覆盖库与 CLI。

## License

MIT
