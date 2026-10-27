//! `litigate-feed` end to end: the repository's packages, a local
//! `CourtListener`, and a `br` that records what it was asked to file.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::too_many_lines, clippy::unnecessary_wraps)]

mod support;

use assert_cmd::Command;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use support::{ok, serve};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litigate-feed-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A `br` appending its arguments (one per line) to `log`, exiting `code`.
fn br(dir: &Path, log: &Path, code: u8) -> PathBuf {
    script(
        &dir.join("br"),
        &format!(
            "for a in \"$@\"; do echo \"$a\" >> '{}'; done\necho refused >&2\nexit {code}\n",
            log.display()
        ),
    )
}

/// The pod's `br` shim: records each call as one `BR_INTENT` line in
/// `log` (the subcommand and its external ref) and touches no store.
fn shim(dir: &Path, log: &Path) -> PathBuf {
    script(
        &dir.join("br-shim"),
        &format!(
            "ref=; prev=\nfor a in \"$@\"; do [ \"$prev\" = --external-ref ] && ref=$a; prev=$a; done\n\
             echo \"BR_INTENT $1 --external-ref $ref\" >> '{}'\n",
            log.display()
        ),
    )
}

fn script(path: &Path, body: &str) -> PathBuf {
    std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_owned()
}

fn feed(base: &str, extra: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("litigate-feed").unwrap();
    cmd.env_remove("COURTLISTENER_API_TOKEN")
        .args(["--repo", concat!(env!("CARGO_MANIFEST_DIR"), "/../..")])
        .args(["--base-url", base, "--min-interval-ms", "0"])
        .args(extra);
    cmd
}

fn ca8(cluster: u64, opinion: u64, case: &str) -> String {
    format!(
        r#"{{"cluster_id": {cluster}, "opinions": [{{"id": {opinion}}}], "court_id": "ca8",
        "dateFiled": "2026-10-26", "caseName": "{case}", "absolute_url": "/opinion/{cluster}/x/"}}"#
    )
}

#[test]
fn a_ca8_opinion_is_filed_with_its_candidate_elements_and_external_ref() {
    let server = serve(vec![
        (
            "/api/rest/v4/search/",
            vec![ok(
                r#"{"results": [{"cluster_id": 1, "opinions": [{"id": 11}], "court_id": "ca8",
                "dateFiled": "2026-10-26", "caseName": "Doe v. Roe", "absolute_url": "/opinion/1/doe-v-roe/"}]}"#,
            )],
        ),
        (
            "/api/rest/v4/opinions/11/",
            vec![ok(
                r#"{"plain_text": "Her notice of appeal was untimely under FRAP 4(a)(1)(A); see FRAP 3(a)(1)."}"#,
            )],
        ),
    ]);
    let dir = scratch("filed");
    let log = dir.join("br.log");
    let program = br(&dir, &log, 0);
    let br_arg = program.to_str().unwrap();
    let out = feed(
        &server.base,
        &["--since", "2026-10-26", "--file", "--br", br_arg],
    )
    .output()
    .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["beads"][0]["court"], "ca8");
    assert_eq!(
        report["beads"][0]["external_ref"],
        "courtlistener:opinion:11"
    );
    let candidates = report["beads"][0]["candidates"].as_array().unwrap();
    assert!(candidates
        .iter()
        .any(|c| c["package"] == "ca8-court-of-appeals" && c["element"] == "notice-filed"));
    let args = std::fs::read_to_string(&log).unwrap();
    assert!(
        args.starts_with("create\nOpinion feed: Doe v. Roe (ca8, 2026-10-26)"),
        "{args}"
    );
    assert!(
        args.contains("--external-ref\ncourtlistener:opinion:11\n"),
        "{args}"
    );
    assert!(args.contains("Opinion: https://www.courtlistener.com/opinion/1/doe-v-roe/"));
    let requests = server.seen.lock().unwrap().clone();
    assert!(
        requests[0]
            .path
            .contains("court=bap8+ca1+ca8+cadc+cafc+uscfc"),
        "{}",
        requests[0].path
    );
    assert!(requests[0].path.contains("filed_after=2026-10-26"));

    // A store refusing the bead fails the run.
    let refusing = br(&dir, &dir.join("refused.log"), 1);
    let out = feed(
        &server.base,
        &["--file", "--br", refusing.to_str().unwrap()],
    )
    .output()
    .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        report["errors"][0].as_str().unwrap().contains("refused"),
        "{report}"
    );

    // A missing `br` is a failure to file, too.
    let out = feed(&server.base, &["--file", "--br", "/nonexistent/br"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("cannot run"));
}

#[test]
fn a_dry_run_lists_the_external_ref_of_every_opinion_and_runs_no_br() {
    let results = format!(
        r#"{{"results": [{}, {}]}}"#,
        ca8(1, 11, "Doe v. Roe"),
        ca8(2, 21, "Poe v. Moe")
    );
    let server = serve(vec![
        ("/api/rest/v4/search/", vec![ok(&results)]),
        (
            "/api/rest/v4/opinions/",
            vec![ok(r#"{"plain_text": "FRAP 4(a)."}"#)],
        ),
    ]);
    let dir = scratch("dry");
    let log = dir.join("br.log");
    let program = br(&dir, &log, 0);
    let out = feed(
        &server.base,
        &["--since", "2026-10-26", "--br", program.to_str().unwrap()],
    )
    .output()
    .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["filed"], false);
    let refs: Vec<&str> = report["beads"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["external_ref"].as_str().unwrap())
        .collect();
    assert_eq!(
        refs,
        ["courtlistener:opinion:11", "courtlistener:opinion:21"]
    );
    assert!(!log.exists());
}

#[test]
fn file_under_the_br_shim_emits_one_create_intent_per_opinion_and_the_same_refs_again() {
    let results = format!(
        r#"{{"results": [{}, {}]}}"#,
        ca8(1, 11, "Doe v. Roe"),
        ca8(2, 21, "Poe v. Moe")
    );
    let server = serve(vec![
        ("/api/rest/v4/search/", vec![ok(&results)]),
        (
            "/api/rest/v4/opinions/",
            vec![ok(r#"{"plain_text": "FRAP 4(a)."}"#)],
        ),
    ]);
    let dir = scratch("shim");
    let (log, work) = (dir.join("intents.log"), dir.join("work"));
    let program = shim(&dir, &log);
    // The run's directory is read-only: the feed writes nothing of its own.
    std::fs::create_dir_all(&work).unwrap();
    std::fs::set_permissions(&work, std::fs::Permissions::from_mode(0o555)).unwrap();
    let run = || {
        let out = feed(
            &server.base,
            &[
                "--since",
                "2026-10-26",
                "--file",
                "--br",
                program.to_str().unwrap(),
            ],
        )
        .current_dir(&work)
        .output()
        .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(report["filed"], true);
        std::fs::read_to_string(&log).unwrap()
    };
    let first = run();
    assert_eq!(
        first,
        "BR_INTENT create --external-ref courtlistener:opinion:11\n\
         BR_INTENT create --external-ref courtlistener:opinion:21\n"
    );
    std::fs::remove_file(&log).unwrap();
    // A second run over the same window proposes the identical refs; the
    // dispatcher's dedupe on external_ref drops them.
    assert_eq!(run(), first);
    assert_eq!(std::fs::read_dir(&work).unwrap().count(), 0);
    std::fs::set_permissions(&work, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn bad_input_fails_before_any_request() {
    let out = feed("http://127.0.0.1:9", &["--since", "yesterday"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--since `yesterday`"));
}
