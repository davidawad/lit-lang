//! `litigate vendor` (lg-70c6): a matter that vendored `lit/core` and
//! `lit/frcp` from an older schema gets L1015 naming the fix, and
//! `vendor` plus `lock` makes `check` pass.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{codes, copy_dir, litigate_in, repo, scratch};

#[test]
fn a_stale_vendored_core_is_l1015_until_vendor_and_lock() {
    let dir = scratch("vendor-stale-core");
    copy_dir(&repo().join("lit/core"), &dir.join("lit/core"));
    copy_dir(&repo().join("lit/frcp"), &dir.join("lit/frcp"));
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::copy(
        repo().join("cases/frcp-civil-case.case"),
        dir.join("cases/frcp-civil-case.case"),
    )
    .unwrap();
    // The schema before lg-60mg: a family listed its forums in core.
    let manifest = dir.join("lit/core/Lit.toml");
    let text = std::fs::read_to_string(&manifest).unwrap();
    let old = text.replacen(
        "[family.district-civil]\n",
        "[family.district-civil]\nforums = [\"frcp\"]\n",
        1,
    );
    assert_ne!(old, text, "core has no [family.district-civil]");
    std::fs::write(&manifest, old).unwrap();

    let stale = litigate_in(&dir, &["check", "cases/frcp-civil-case.case"]);
    let l1015 = stale.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "L1015")
        .unwrap_or_else(|| panic!("no L1015: {}", stale.stdout));
    let message = l1015["message"].as_str().unwrap();
    assert!(
        message.contains("vendored `core`") && message.contains("litigate vendor lit"),
        "{message}"
    );
    assert!(!codes(&stale.json).iter().any(|(c, _)| c == "L1004"));

    let vendor = litigate_in(&dir, &["vendor", "lit"]);
    assert_eq!(vendor.code, 0, "{}{}", vendor.stdout, vendor.stderr);
    assert!(dir.join("lit/core/.litigate-vendor").is_file());
    for target in ["lit/frcp", "cases"] {
        let lock = litigate_in(&dir, &["lock", target]);
        assert_eq!(lock.code, 0, "{}{}", lock.stdout, lock.stderr);
    }
    let check = litigate_in(&dir, &["check", "cases/frcp-civil-case.case"]);
    let errors: Vec<_> = codes(&check.json)
        .into_iter()
        .filter(|(_, s)| s == "error")
        .collect();
    assert_eq!(errors, [], "{}", check.stdout);
    assert_eq!(check.json["ok"], true);
}
