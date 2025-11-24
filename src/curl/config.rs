//! Centralized curl option metadata used by the parser. Update this file when
//! you need to support new aliases or value requirements.

/// Flags that behave as method specifiers and require a payload.
pub const METHOD_FLAG_IDENTIFIERS: [&str; 2] = ["-X", "--request"];

/// Header flags, always followed by header contents.
pub const HEADER_FLAG_IDENTIFIERS: [&str; 2] = ["-H", "--header"];

/// Flags that should be treated as payload/data modifiers.
pub const DATA_FLAG_IDENTIFIERS: [&str; 8] = [
    "--data-urlencode",
    "--data-binary",
    "--data-raw",
    "--data",
    "-d",
    "--form-string",
    "--form",
    "-F",
];

/// Flags that must be followed by a value. Keep alphabetically sorted.
pub const FLAG_VALUE_REQUIRED: [&str; 18] = [
    "--cacert",
    "--cert",
    "--cert-type",
    "--connect-timeout",
    "--cookie",
    "--cookie-jar",
    "--key",
    "--key-type",
    "--limit-rate",
    "--max-time",
    "--output",
    "--proxy",
    "--retry",
    "--retry-delay",
    "--retry-max-time",
    "--trace",
    "--trace-ascii",
    "--user",
];

/// Short flags that also must be followed by a value.
pub const SHORT_FLAGS_VALUE_REQUIRED: [&str; 3] = ["-o", "-u", "-x"];
