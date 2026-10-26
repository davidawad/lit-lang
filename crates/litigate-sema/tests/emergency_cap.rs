//! The cap on an emergency declaration's grants is the vendored rule's:
//! `lit/core/Lit.toml` bounds the district-civil `emergency-declaration`
//! cell at the period FRCP 87(b)(1)(C) states in `sources/frcp.txt`
//! (L2011 enforces it, `change.rs`).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

fn read(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read_to_string(root.join(relative)).expect(relative)
}

/// The number of days in `text` after `phrase`, whitespace normalized.
fn days_after(text: &str, phrase: &str) -> u32 {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let rest = flat[flat.find(phrase).expect(phrase) + phrase.len()..].trim_start();
    let n = rest.split(' ').next().expect("a number");
    assert!(rest[n.len()..].starts_with(" days"), "{rest:.40}");
    n.parse().expect("a number")
}

#[test]
fn the_district_civil_cap_is_frcp_87s_vendored_period() {
    let frcp = read("sources/frcp.txt");
    let rule = &frcp[frcp.find("## Rule 87\n").expect("Rule 87 is vendored")..];
    let rule = &rule[..rule
        .find("(As added Apr. 24, 2023, eff. Dec. 1, 2023.)")
        .expect("in force")];
    let vendored = days_after(rule, "must be limited to a stated period of no more than");
    let toml = read("lit/core/Lit.toml");
    let cell = &toml[toml
        .find("[family.district-civil.class.emergency-declaration]")
        .expect("the cell")..];
    let cell = &cell[..cell[1..].find("\n[").map_or(cell.len(), |i| i + 1)];
    let window: u32 = cell
        .lines()
        .find_map(|l| l.strip_prefix("window_days = "))
        .expect("window_days")
        .trim()
        .parse()
        .expect("a number");
    assert_eq!((vendored, window), (90, 90));
}
