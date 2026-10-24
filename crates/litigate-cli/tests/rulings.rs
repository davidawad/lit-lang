//! The five decisions of the 2026-10-23 ruling (lg-litigate-epic-tuq.52),
//! each pinned on the real packages and the repository's own build: an
//! event after the matter's `as_of` is rejected (SPEC §5.6); a missed
//! deadline is covered only by a cited event of an excusing type that an
//! `after_expiry` grant admits (§5.6); the shared clock library is
//! hand-written (§9.4); the sync build runs under the `dev` profile
//! (§9.4); and the earlier frcp fee versions are known from the day they
//! were recorded (§5.4).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate, litigate_in, repo, scratch, Run};

/// `litigate check` of a matter on `lit/frcp` at `as_of` with `events`.
fn frcp(name: &str, as_of: &str, events: &[&str]) -> Run {
    let dir = scratch(name);
    for pkg in ["core", "frcp"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    // The copied locks pin the vendored L0 texts (SPEC §4.2).
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    std::fs::write(
        dir.join("cases/ruling.case"),
        format!(
            "case ruling\nuses frcp-civil-procedure ^2024.12.0\nas_of {as_of}\nsummary \"s\"\n\n\
             parties {{\n  plaintiff: applicant Corporation\n  judge: court Tribunal\n}}\n\n\
             events {{\n{events}}}\n\n\
             stakes {{ frcp-civil-procedure::appeal-window-blown: 0.usd }}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    litigate_in(&dir, &["check", "cases/ruling.case"])
}

/// `(code, message)` of every error.
fn errors(run: &Run) -> Vec<(String, String)> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_owned(),
                d["message"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// `(status, extended_to)` of each instance of frcp's service period.
fn service_period(run: &Run) -> Vec<(String, String)> {
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["deadline"] == "frcp-civil-procedure::service-period")
        .map(|o| {
            (
                o["status"].as_str().unwrap().to_owned(),
                o["extended_to"].as_str().unwrap_or("-").to_owned(),
            )
        })
        .collect()
}

const FILED: &str = "2025-01-06 ComplaintFiled() by plaintiff";

#[test]
fn an_event_after_as_of_is_rejected() {
    let late = frcp(
        "ruling-after-as-of",
        "2025-06-01",
        &[
            FILED,
            "2025-06-02 ExtensionMoved(ServicePeriod) by plaintiff",
        ],
    );
    assert_eq!(late.code, 2, "{}", late.stdout);
    assert_eq!(
        errors(&late),
        [(
            "L4001".to_owned(),
            "event on 2025-06-02 is after the matter's as_of 2025-06-01".to_owned()
        )]
    );
    // An event dated on `as_of` itself is history.
    let on = frcp(
        "ruling-on-as-of",
        "2025-06-02",
        &[
            FILED,
            "2025-06-02 ExtensionMoved(ServicePeriod) by plaintiff",
        ],
    );
    assert_eq!(on.code, 0, "{}", on.stdout);
    assert_eq!(errors(&on), []);
}

#[test]
fn a_cited_excusing_event_covers_a_missed_deadline() {
    // The Rule 4(m) period ran out on 2025-04-07. A motion made after it
    // and a cited order of the excusing type `ServicePeriodExtended`
    // (`extends service-period`, FRCP 6(b)(1)(B)'s `after_expiry` grant)
    // cover it.
    let covered = frcp(
        "ruling-covered",
        "2025-06-01",
        &[
            FILED,
            "2025-04-20 ExtensionMoved(ServicePeriod) by plaintiff",
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite \"FRCP 6(b)(1)(B)\"",
        ],
    );
    assert_eq!(covered.code, 0, "{}", covered.stdout);
    assert_eq!(errors(&covered), []);
    assert_eq!(
        service_period(&covered),
        [("open".to_owned(), "2026-05-01".to_owned())]
    );

    // A cited event with no declared effect covers nothing: the period
    // stays missed.
    let named = frcp(
        "ruling-cite-alone",
        "2025-06-01",
        &[
            FILED,
            "2025-04-20 ExtensionMoved(ServicePeriod) by plaintiff cite \"FRCP 6(b)(1)(B)\"",
        ],
    );
    assert_eq!(named.code, 0, "{}", named.stdout);
    assert_eq!(
        service_period(&named),
        [("lapsed".to_owned(), "-".to_owned())]
    );

    // An excusing event without a cite is rejected.
    let uncited = frcp(
        "ruling-uncited",
        "2025-06-01",
        &[
            FILED,
            "2025-04-20 ExtensionMoved(ServicePeriod) by plaintiff",
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge",
        ],
    );
    assert_eq!(uncited.code, 2, "{}", uncited.stdout);
    assert!(
        errors(&uncited).iter().all(|(code, _)| code == "L4001") && !errors(&uncited).is_empty(),
        "{:?}",
        errors(&uncited)
    );
}

/// The `litigate build` arguments `crates/litgraph-packs` and `just
/// lit-sync` build the repository from.
fn build_args() -> Vec<String> {
    std::fs::read_to_string(repo().join("crates/litgraph-packs/build.args"))
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// `litigate build <build.args> [extra] --out <scratch>` at the repository
/// root; the run and its output directory.
fn build(name: &str, extra: &[&str]) -> (Run, PathBuf) {
    let out = scratch(name);
    let mut args: Vec<String> = vec!["build".into()];
    args.extend(extra.iter().map(|s| (*s).to_owned()));
    args.extend(build_args());
    args.extend(["--out".to_owned(), out.display().to_string()]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    (litigate(&args), out)
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// `(file name, path)` of every `*.json` directly in `dir`, sorted.
fn json_files(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter(|p| !p.to_string_lossy().ends_with(".map.json"))
        .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), p))
        .collect();
    files.sort();
    files
}

#[test]
fn the_shared_clock_library_is_hand_written_and_sync_builds_under_dev() {
    // Sync names no profile, so `build` runs under its default, `dev`.
    assert!(
        !build_args().iter().any(|a| a.starts_with("--profile")),
        "build.args must not choose a profile"
    );
    let justfile = std::fs::read_to_string(repo().join("justfile")).unwrap();
    let sync = justfile
        .split("\nlit-sync:")
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap();
    assert!(sync.contains(" build "), "{sync}");
    assert!(!sync.contains("--profile"), "{sync}");

    let (dev, out) = build("ruling-build-dev", &[]);
    assert_eq!(dev.code, 0, "{}{}", dev.stdout, dev.stderr);
    let litigatables = json_files(&out.join("litigatables"));
    assert!(!litigatables.is_empty());
    for (name, path) in &litigatables {
        assert_eq!(json(path)["verification"]["profile"], "dev", "{name}");
    }

    // Every committed clock file is a hand-written schema 3 fragment, and
    // the build generates only schema 4 fragments, under other names.
    let hand = json_files(&repo().join("packs/clock"));
    assert_eq!(
        hand.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        [
            "frap26.json",
            "frcp6.json",
            "frcrimp45.json",
            "itc210.json",
            "rcfc6.json",
            "sup-ct-30.json",
            "us-federal-holidays.json",
        ]
    );
    for (name, path) in &hand {
        let doc = json(path);
        assert_eq!(doc["schemaVersion"], 3, "{name}");
        assert!(doc.get("lowered").is_none(), "{name}");
    }
    let generated = json_files(&out.join("packs/clock"));
    assert!(!generated.is_empty());
    for (name, path) in &generated {
        assert!(hand.iter().all(|(h, _)| h != name), "{name} is generated");
        let doc = json(path);
        assert_eq!(doc["schemaVersion"], 4, "{name}");
        assert!(doc.get("lowered").is_some(), "{name}");
    }

    // Under `strict` the same sources do not build: their fail-closed
    // holes are errors there, so a sync under `strict` would block on
    // them or invite filling them in.
    let (strict, _) = build("ruling-build-strict", &["--profile", "strict"]);
    assert_eq!(strict.code, 2, "{}", strict.stderr);
    assert!(errors(&strict).iter().any(|(code, _)| code == "L5002"));
}

/// `(from, to, known_at)` of each version `litigate where` reports for `id`.
fn versions(id: &str) -> Vec<(String, String, String)> {
    let run = litigate(&["where", id]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let field = |v: &Value| v.as_str().unwrap_or("..").to_owned();
    let mut out: Vec<_> = run.json["result"]["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                field(&v["in_force"]["from"]),
                field(&v["in_force"]["to"]),
                field(&v["known_at"]),
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn the_earlier_frcp_fee_versions_are_known_from_their_recording() {
    // The $505 versions were recorded 2026-10-22, after the package's
    // `known_at` (2024-12-01) that the $605 versions inherit.
    for id in [
        "frcp-civil-procedure::notice-of-appeal-window",
        "frcp-civil-procedure::cross-appeal-window",
    ] {
        assert_eq!(
            versions(id),
            [
                (
                    "2013-12-01".to_owned(),
                    "2023-12-01".to_owned(),
                    "2026-10-22".to_owned()
                ),
                (
                    "2023-12-01".to_owned(),
                    "..".to_owned(),
                    "2024-12-01".to_owned()
                ),
            ],
            "{id}"
        );
    }
}
