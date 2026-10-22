//! Spelling legacy JSON values as `.lit`/`.case` tokens (SPEC §2).
//!
//! Every function here is exact: it either spells the value so that
//! lowering gives back the same binary64 / string, or returns `None` and
//! the caller reports L6002. Numbers are spelled from the shortest decimal
//! that round-trips the `f64` (Rust's `Display`), which SPEC §8.5 reads back
//! as an exact rational and rounds to the same bits.

/// A string literal: quotes plus the escapes `\\ \" \n \r \t` (SPEC §2).
#[must_use]
pub fn string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Whether `text` lexes as one identifier token (SPEC §2): letters, digits
/// and `_`, hyphens only between two of those, starting with a letter or
/// `_` (or digits then a letter). The reserved word `cite` and a lone `_`
/// are not identifiers.
#[must_use]
pub fn is_ident(text: &str) -> bool {
    let continues = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let starts = text
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .chars()
        .next();
    let leading_ok = match text.chars().next() {
        Some(c) if c.is_ascii_digit() => starts.is_some_and(|c| c.is_ascii_alphabetic()),
        Some(c) => c.is_ascii_alphabetic() || c == '_',
        None => false,
    };
    leading_ok
        && text != "_"
        && text != "cite"
        && text
            .split('-')
            .all(|part| !part.is_empty() && part.chars().all(continues))
}

/// A reference `a::b` / `a@b::c` whose every segment is an identifier.
#[must_use]
pub fn is_ref(text: &str) -> bool {
    text.split("::").all(|segment| {
        let mut parts = segment.splitn(2, '@');
        let head = parts.next().unwrap_or_default();
        is_ident(head) && parts.next().is_none_or(is_ident)
    })
}

/// A finite `f64` as the shortest round-tripping decimal, with `-` for
/// negatives (`SignedNumber`). `None` for a non-finite value.
#[must_use]
pub fn number(value: f64) -> Option<String> {
    value.is_finite().then(|| {
        let text = format!("{value}");
        // `-0` has no distinct spelling worth keeping; it equals `0`.
        if text == "-0" {
            "0".to_owned()
        } else {
            text
        }
    })
}

/// Digits grouped by thousands with `_` (`2_000_000`), as SPEC §2 allows.
fn grouped(digits: &str) -> String {
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);
    for (index, c) in digits.chars().enumerate() {
        if index > 0 && (len - index).is_multiple_of(3) {
            out.push('_');
        }
        out.push(c);
    }
    out
}

/// Money in USD (`-2_000_000.usd`, `12.5.usd`). `None` when the amount
/// isn't a whole number of cents, so it has no exact `usd` spelling.
#[must_use]
pub fn money(value: f64) -> Option<String> {
    let text = number(value)?;
    let (sign, magnitude) = text
        .strip_prefix('-')
        .map_or(("", text.as_str()), |rest| ("-", rest));
    let (whole, fraction) = magnitude.split_once('.').unwrap_or((magnitude, ""));
    let digits_only = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    if fraction.len() > 2 || !digits_only(whole) || !digits_only(fraction) {
        return None;
    }
    let fraction = if fraction.is_empty() {
        String::new()
    } else {
        format!(".{fraction}")
    };
    Some(format!("{sign}{}{fraction}.usd", grouped(whole)))
}

/// A non-negative duration literal (`21.days`, `4.5.days`, `0.5.hours`).
/// `whole` demands an integral amount. `None` when that fails or the
/// amount is negative or non-finite.
#[must_use]
pub fn duration(value: f64, unit: &str, whole: bool) -> Option<String> {
    let text = number(value)?;
    let plain = text.chars().all(|c| c.is_ascii_digit() || c == '.');
    (plain && !(whole && text.contains('.'))).then(|| format!("{text}.{unit}"))
}

/// The `cite` clause spelling of a free-text citation: the text itself
/// when the lexer's citation mode gives it back unchanged, else a cite hole
/// carrying the original text as its reason (SPEC §2, §8.1).
#[must_use]
pub fn cite(text: &str, recognized: bool) -> String {
    if recognized && is_cite_text(text) {
        format!("cite {text}")
    } else {
        format!("cite ?({})", string(text))
    }
}

/// Whether citation mode reads `text` back verbatim: one line, no `{ } ,`
/// or `//`, no leading `?`, and no whitespace that the lexer would trim or
/// collapse.
#[must_use]
pub fn is_cite_text(text: &str) -> bool {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    !text.is_empty()
        && collapsed == text
        && !text.starts_with('?')
        && !text.contains("//")
        && !text.contains(['{', '}', ',', '\n'])
}

#[cfg(test)]
mod tests;
