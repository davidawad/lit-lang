//! `docs/litigate/TUTORIAL.md` stays true: its `Lit.toml`, rule module,
//! tests and case are extracted verbatim, laid out as the tutorial says
//! (beside a copy of `lit/core`), and every claim it makes about `check`,
//! `test`, `run` and `build` is asserted.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::json;
use support::{copy_dir, litigate_in, repo, scratch};

/// The bodies of the tutorial's fenced blocks tagged `lang`, in order.
fn blocks(text: &str, lang: &str) -> Vec<String> {
    let fence = format!("```{lang}\n");
    text.split(fence.as_str())
        .skip(1)
        .map(|rest| rest.split("```").next().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn the_tutorial_runs_as_written() {
    let text = std::fs::read_to_string(repo().join("docs/litigate/TUTORIAL.md")).unwrap();
    let (toml, lit) = (blocks(&text, "toml"), blocks(&text, "lit"));
    let [manifest] = toml.as_slice() else {
        panic!("one toml block expected, found {}", toml.len())
    };
    let [rules, tests, case] = lit.as_slice() else {
        panic!("three lit blocks expected, found {}", lit.len())
    };
    let root = scratch("tutorial");
    copy_dir(&repo().join("lit/core"), &root.join("lit/core"));
    let pkg = root.join("lit/my-court");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::create_dir_all(root.join("cases")).unwrap();
    std::fs::write(pkg.join("Lit.toml"), manifest).unwrap();
    std::fs::write(pkg.join("answer.lit"), format!("{rules}\n{tests}")).unwrap();
    std::fs::write(root.join("cases/acme-v-roadrunner.case"), case).unwrap();
    let run = |args: &[&str]| litigate_in(&root, args);

    for lock in ["lit/my-court", "cases"] {
        let locked = run(&["lock", lock]);
        assert_eq!(locked.code, 0, "{}", locked.stdout);
    }
    // The tutorial's blocks are already canonical: `fmt --check` passes.
    let fmt = run(&["fmt", "--check", "lit/my-court", "cases"]);
    assert_eq!(fmt.code, 0, "{}", fmt.stdout);
    let strict = run(&["check", "--profile", "strict", "lit/my-court", "cases"]);
    assert_eq!(strict.code, 0, "{}", strict.stdout);
    assert_eq!(strict.json["diagnostics"], json!([]));

    let tested = run(&["test", "lit/my-court"]);
    assert_eq!(
        tested.json["result"]["passed"],
        json!(3),
        "{}",
        tested.stdout
    );
    assert_eq!(tested.json["result"]["failed"], json!(0));
    let steps = tested.json["result"]["tests"][0]["expectations"][0]["steps"][0].clone();
    assert_eq!(
        steps,
        json!("FRCP 6(a): exclude the trigger date 2025-03-03")
    );

    let solved = run(&["run", "cases/acme-v-roadrunner.case", "--op", "solve"]);
    let result = &solved.json["result"]["result"];
    assert_eq!(result["value"], json!(34800.0), "{}", solved.stdout);
    assert_eq!(result["best_line"][0]["id"], json!("my-court::answer"));
    assert_eq!(result["best_line"][0]["deadline_days"], json!(21.0));
    assert_eq!(result["best_line"][1]["p"], json!(0.6));

    let built = run(&[
        "build",
        "lit/my-court",
        "cases/acme-v-roadrunner.case",
        "--out",
        "build",
    ]);
    assert_eq!(built.code, 0, "{}", built.stdout);
    for file in [
        "packs/my-court.json",
        "packs/my-court.map.json",
        "scenarios/acme-v-roadrunner.json",
        "litigatables/my-court.litigatable.json",
    ] {
        assert!(root.join("build").join(file).is_file(), "{file}");
    }
    let found = run(&["where", "my-court::answer", "--path", "lit/my-court"]);
    assert_eq!(found.json["ok"], json!(true), "{}", found.stdout);

    // The failing-test example: one wrong date is L5006 naming the right one.
    let wrong = format!("{rules}\n{}", tests.replace("2025-03-31", "2025-03-29"));
    std::fs::write(pkg.join("answer.lit"), wrong).unwrap();
    let failed = run(&["test", "lit/my-court"]);
    assert_eq!(failed.code, 2);
    let message = failed.json["diagnostics"][0]["message"]
        .as_str()
        .unwrap_or_default();
    assert!(text.contains(message), "tutorial must quote: {message}");
}
