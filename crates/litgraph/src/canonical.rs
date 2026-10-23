// SPDX-License-Identifier: GPL-3.0-or-later
//! Canonical JSON: the JSON Canonicalization Scheme (JCS, RFC 8785).
//!
//! [`to_jcs`] is the exact canonical text that digests are taken over:
//! object members sorted by the UTF-16 code units of their names, no
//! insignificant whitespace, strings escaped minimally (RFC 8785 §3.2.2.2)
//! and numbers written as ECMAScript's `Number.prototype.toString` writes
//! the IEEE 754 double they denote (§3.2.2.3). [`to_jcs_pretty`] is the same
//! text with two-space indentation and a trailing newline, the form
//! generated files are committed in: deleting its insignificant whitespace
//! gives [`to_jcs`] back byte for byte.
//!
//! ```
//! use litgraph::canonical::{to_jcs, to_jcs_pretty};
//! let v = serde_json::json!({"b": [1.0, 1e21, 0.000001], "a": "\u{1f}"});
//! assert_eq!(to_jcs(&v), r#"{"a":"\u001f","b":[1,1e+21,0.000001]}"#);
//! assert_eq!(to_jcs_pretty(&v).lines().count(), 8);
//! ```

use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// The JCS text of `value`.
#[must_use]
pub fn to_jcs(value: &Value) -> String {
    let mut out = String::new();
    write(&mut out, value, None);
    out
}

/// The JCS text of `value`, indented by two spaces per level, with a
/// trailing newline.
#[must_use]
pub fn to_jcs_pretty(value: &Value) -> String {
    let mut out = String::new();
    write(&mut out, value, Some(0));
    out.push('\n');
    out
}

/// `sha256:<lowercase hex>` of the JCS text of `value`.
#[must_use]
pub fn digest(value: &Value) -> String {
    Sha256::digest(to_jcs(value).as_bytes())
        .iter()
        .fold(String::from("sha256:"), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// Appends `value`; `indent` is the pretty-printing depth (`None`: compact).
fn write(out: &mut String, value: &Value, indent: Option<usize>) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&number(n)),
        Value::String(s) => string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                separate(out, i, indent);
                write(out, item, indent.map(|d| d + 1));
            }
            close(out, items.is_empty(), indent, ']');
        }
        Value::Object(map) => {
            out.push('{');
            for (i, (key, item)) in sorted(map).into_iter().enumerate() {
                separate(out, i, indent);
                string(out, key);
                out.push(':');
                if indent.is_some() {
                    out.push(' ');
                }
                write(out, item, indent.map(|d| d + 1));
            }
            close(out, map.is_empty(), indent, '}');
        }
    }
}

/// The comma before member `i` and, pretty, its line break and indentation.
fn separate(out: &mut String, i: usize, indent: Option<usize>) {
    if i > 0 {
        out.push(',');
    }
    if let Some(depth) = indent {
        out.push('\n');
        out.push_str(&"  ".repeat(depth + 1));
    }
}

/// The closing bracket, on its own line when pretty and non-empty.
fn close(out: &mut String, empty: bool, indent: Option<usize>, bracket: char) {
    if let (Some(depth), false) = (indent, empty) {
        out.push('\n');
        out.push_str(&"  ".repeat(depth));
    }
    out.push(bracket);
}

/// Members in RFC 8785 §3.2.3 order: by the UTF-16 code units of the name.
fn sorted(map: &Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut members: Vec<(&String, &Value)> = map.iter().collect();
    members.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
    members
}

/// A JSON string per RFC 8785 §3.2.2.2: `"` and `\` escaped, the control
/// characters as their short escapes or `\u00xx`, everything else verbatim.
fn string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// The largest integer every smaller one of which a double holds exactly.
const SAFE: u64 = 1 << 53;

/// A JSON number as the double it denotes, per RFC 8785 §3.2.2.3.
fn number(n: &Number) -> String {
    match (n.as_u64(), n.as_i64()) {
        (Some(u), _) if u <= SAFE => u.to_string(),
        (_, Some(i)) if i.unsigned_abs() <= SAFE => i.to_string(),
        _ => double(n.as_f64().unwrap_or(0.0)),
    }
}

/// ECMAScript `Number::toString(x)` for a finite double (ECMA-262
/// §6.1.6.1.20): the shortest round-tripping digits `s` (`k` of them) and
/// the exponent `n` with `x = s × 10^(n−k)`, laid out in plain decimal for
/// `−6 < n ≤ 21` and in exponent form otherwise.
#[must_use]
pub fn double(x: f64) -> String {
    if x == 0.0 {
        return "0".to_owned();
    }
    if x < 0.0 {
        return format!("-{}", double(-x));
    }
    // Rust's `{:e}` is the shortest round-tripping `d.ddde±x`.
    let sci = format!("{x:e}");
    let (mantissa, exponent) = sci.split_once('e').unwrap_or((&sci, "0"));
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let k = i64::try_from(digits.len()).unwrap_or(i64::MAX);
    let n = exponent.parse::<i64>().unwrap_or(0) + 1;
    let zeros = |count: i64| "0".repeat(usize::try_from(count).unwrap_or(0));
    if k <= n && n <= 21 {
        format!("{digits}{}", zeros(n - k))
    } else if 0 < n && n <= 21 {
        let (int, frac) = digits.split_at(usize::try_from(n).unwrap_or(0));
        format!("{int}.{frac}")
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", zeros(-n))
    } else {
        let e = n - 1;
        let sign = if e < 0 { '-' } else { '+' };
        let (first, rest) = digits.split_at(1);
        let point = if rest.is_empty() { "" } else { "." };
        format!("{first}{point}{rest}e{sign}{}", e.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_follow_ecmascript() {
        // RFC 8785 Appendix B, plus the layout boundaries.
        let cases: [(f64, &str); 18] = [
            (0.0, "0"),
            (-0.0, "0"),
            (1.0, "1"),
            (-1.5, "-1.5"),
            (1e21, "1e+21"),
            (1e20, "100000000000000000000"),
            (123_456_789_012_345_680_000.0, "123456789012345680000"),
            (1.5e21, "1.5e+21"),
            (1e-6, "0.000001"),
            (1e-7, "1e-7"),
            (1.25e-7, "1.25e-7"),
            (0.1, "0.1"),
            (12.5, "12.5"),
            (5e-324, "5e-324"),
            (1.797_693_134_862_315_7e308, "1.7976931348623157e+308"),
            (9_007_199_254_740_992.0, "9007199254740992"),
            (295_147_905_179_352_830_000.0, "295147905179352830000"),
            (0.000_001_234, "0.000001234"),
        ];
        for (x, want) in cases {
            assert_eq!(double(x), want, "{x:e}");
        }
    }

    #[test]
    fn integers_beyond_two_to_the_53_are_written_as_doubles() {
        let v = json!([9_007_199_254_740_993_u64, -9_007_199_254_740_993_i64, -3, 7]);
        assert_eq!(to_jcs(&v), "[9007199254740992,-9007199254740992,-3,7]");
    }

    #[test]
    fn strings_escape_minimally() {
        let v = json!("\"\\\u{8}\u{c}\n\r\t\u{1}\u{7f}é€/");
        assert_eq!(to_jcs(&v), "\"\\\"\\\\\\b\\f\\n\\r\\t\\u0001\u{7f}é€/\"");
    }

    #[test]
    fn members_sort_by_utf16_code_units() {
        // RFC 8785 §3.2.3: U+E000 sorts after U+1F600 in UTF-8 but before it
        // in UTF-16 (a surrogate pair starts 0xD83D).
        let v = json!({"\u{e000}": 1, "\u{1f600}": 2, "b": [true, false, null], "a": {}});
        assert_eq!(
            to_jcs(&v),
            "{\"a\":{},\"b\":[true,false,null],\"\u{1f600}\":2,\"\u{e000}\":1}"
        );
    }

    #[test]
    fn pretty_is_jcs_plus_whitespace() {
        let v = json!({"z": [], "y": [1, {"b": 2, "a": [3]}], "x": "a b"});
        let pretty = to_jcs_pretty(&v);
        assert_eq!(
            pretty,
            "{\n  \"x\": \"a b\",\n  \"y\": [\n    1,\n    {\n      \"a\": [\n        3\n      ],\n      \"b\": 2\n    }\n  ],\n  \"z\": []\n}\n"
        );
        let squeezed: String = pretty
            .lines()
            .map(|l| l.trim_start().replacen("\": ", "\":", 1))
            .collect();
        assert_eq!(squeezed, to_jcs(&v));
    }

    #[test]
    fn digest_is_sha256_of_the_jcs_text() {
        assert_eq!(
            digest(&json!({})),
            "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
        );
    }
}
