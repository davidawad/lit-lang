use litigate_engine::cite::SourceCorpus;
use std::fmt::Write as _;

/// Whether `pinpoint` (as `text`, the cite) names a subdivision of the
/// section `heading` in the corpus file `path`.
#[must_use]
pub fn subdivided(
    corpus: &SourceCorpus,
    path: &str,
    heading: &str,
    pin: &[String],
    text: &str,
) -> bool {
    (0..=pin.len()).rev().any(|k| {
        let heading = pin[..k].iter().fold(heading.to_owned(), |mut h, m| {
            let _ = write!(h, "({m})");
            h
        });
        let body: Vec<&str> = corpus
            .lookup_all(&heading)
            .iter()
            .filter(|(file, _)| file == path)
            .map(|(_, section)| section.body.as_str())
            .collect();
        !body.is_empty() && pinpointed(&body.join("\n"), &pin[k..], text)
    })
}

fn pinpointed(body: &str, pinpoint: &[String], text: &str) -> bool {
    let plain = |m: &str| !m.is_empty() && m.len() <= 5 && m.chars().all(char::is_alphanumeric);
    let mut written = Vec::new();
    for line in body.lines() {
        let t = line.trim_start();
        if let Some((m, _)) = t.split_once(['.', ')']).filter(|(m, _)| plain(m)) {
            written.push(m);
        }
        let mut rest = t;
        while let Some((_, after)) = rest.split_once('(') {
            if let Some((m, _)) = after.split_once(')').filter(|(m, _)| plain(m)) {
                written.push(m);
            }
            rest = after;
        }
    }
    if written.is_empty() {
        return true;
    }
    let mut written = written.into_iter();
    pinpoint
        .iter()
        .flat_map(|p| {
            let parts: Vec<&str> = p
                .split(['(', ')'])
                .map(|m| m.trim_matches(|c: char| c == ',' || c == '-' || c.is_whitespace()))
                .filter(|m| !m.is_empty())
                .collect();
            let markers = parts.iter().all(|m| plain(m));
            parts.into_iter().filter(move |_| markers)
        })
        .filter(|m| text.contains(&format!("({m})")))
        .all(|m| written.any(|w| w == m))
}
