//! Times lexing, parsing, and formatting a ~1 MiB synthetic corpus.
use std::time::Instant;
fn main() {
    let example = std::fs::read_to_string("lit/examples/rule12.lit").unwrap_or_default();
    let items: String = example
        .lines()
        .filter(|l| !l.starts_with("module") && !l.starts_with("package"))
        .fold(String::new(), |mut out, l| {
            out.push_str(l);
            out.push('\n');
            out
        });
    let mut text = String::from("module m\n");
    while text.len() < (1 << 20) {
        text.push_str(&items);
    }
    for _ in 0..3 {
        let t = Instant::now();
        let (tokens, _) = litigate_syntax::lexer::lex(&text);
        let lex = t.elapsed();
        let t = Instant::now();
        let parsed = litigate_syntax::parse_lit(&text);
        let parse = t.elapsed();
        let t = Instant::now();
        let out =
            litigate_syntax::format(&text, litigate_syntax::FileKind::Lit).unwrap_or_default();
        let fmt = t.elapsed();
        println!(
            "tokens {} lex {lex:?} parse(total) {parse:?} fmt {fmt:?} ok={} same={}",
            tokens.len(),
            parsed.is_ok(),
            out == text
        );
    }
}
