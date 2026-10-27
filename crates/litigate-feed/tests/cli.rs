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
    let path = dir.join("br");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do echo \"$a\" >> '{}'; done\necho refused >&2\nexit {code}\n",
            log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn feed(base: &str, state: &Path, extra: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("litigate-feed").unwrap();
    cmd.env_remove("COURTLISTENER_API_TOKEN")
        .args(["--repo", concat!(env!("CARGO_MANIFEST_DIR"), "/../..")])
        .args(["--base-url", base, "--min-interval-ms", "0"])
        .arg("--state")
        .arg(state)
        .args(extra);
    cmd
}

#[test]
fn a_ca8_opinion_is_filed_once_with_its_candidate_elements() {
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
    let (state, log) = (dir.join("state.json"), dir.join("br.log"));
    let program = br(&dir, &log, 0);
    let br_arg = program.to_str().unwrap();
    let out = feed(
        &server.base,
        &state,
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
        args.contains("--external-ref\ncourtlistener-opinion-11\n"),
        "{args}"
    );
    assert!(args.contains("Opinion: https://www.courtlistener.com/opinion/1/doe-v-roe/"));
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&state).unwrap()).unwrap();
    assert!(saved["seen"]["11"].is_string());
    assert!(saved["last_poll"].is_string());
    let requests = server.seen.lock().unwrap().clone();
    assert!(
        requests[0]
            .path
            .contains("court=bap8+ca1+ca8+cadc+cafc+uscfc"),
        "{}",
        requests[0].path
    );
    assert!(requests[0].path.contains("filed_after=2026-10-26"));

    // Again from the recorded state: the opinion is already filed.
    std::fs::remove_file(&log).unwrap();
    let out = feed(&server.base, &state, &["--file", "--br", br_arg])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["already_filed"], serde_json::json!([1]));
    assert!(!log.exists());

    // A dry run prints the bead and writes no state.
    let fresh = dir.join("fresh.json");
    let out = feed(&server.base, &fresh, &[]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        (
            report["filed"].as_bool(),
            report["beads"].as_array().map(Vec::len)
        ),
        (Some(false), Some(1))
    );
    assert!(!fresh.exists());

    // A store refusing the bead fails the run and records nothing.
    let refusing = br(&dir, &dir.join("refused.log"), 1);
    let out = feed(
        &server.base,
        &fresh,
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
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&fresh).unwrap()).unwrap();
    assert_eq!(saved["seen"], serde_json::json!({}));

    // A missing `br` is a failure to file, too.
    let out = feed(
        &server.base,
        &dir.join("none.json"),
        &["--file", "--br", "/nonexistent/br"],
    )
    .output()
    .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("cannot run"));
}

#[test]
fn bad_input_fails_before_any_request() {
    let dir = scratch("bad");
    let out = feed(
        "http://127.0.0.1:9",
        &dir.join("s.json"),
        &["--since", "yesterday"],
    )
    .output()
    .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--since `yesterday`"));
    std::fs::write(dir.join("garbled.json"), "not json").unwrap();
    let out = feed("http://127.0.0.1:9", &dir.join("garbled.json"), &[])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("garbled.json"));
}
