//! `.case` stakes reach every state-flag copy of a terminal (lg-ytio).
//! A terminal reached under a flag compiles to a copy `won{amended}`; the
//! matter's `stakes` name the terminal once, so they must value (and
//! fee-shift) each copy, not only the unflagged one, or the solve falls
//! back to a heuristic or zero payoff wherever a flag was set. The
//! synthetic model is a Rule 12(b)(6) dismissal with leave to amend:
//! amend, or stand on the complaint and take the dismissal up on appeal
//! (WMX Techs., Inc. v. Miller, 104 F.3d 1133 (9th Cir. 1997) (en banc)).
//! The numbers are illustrative, not calibrated.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{litigate_in, scratch};

/// The dismissal package; each option sets a flag when `flagged`, and
/// nothing downstream reads it, so the flags change no odds and no cost.
/// No terminal authors a payoff: the matter's stakes are the only values.
fn package(flagged: bool) -> String {
    let (amend, stand) = if flagged {
        (", sets [amended]", ", sets [stood]")
    } else {
        ("", "")
    };
    format!(
        r#"module demo.dismissal

package dismissal {{ forum frcp, authority FederalRule }}

start dismissed

decision dismissed: "Dismissed with leave to amend" by Plaintiff {{
  amend "File an amended complaint" -> ruling {{ cost 400.usd{amend} }}
  stand "Stand on the complaint and appeal" -> ruling {{ cost 600.usd{stand} }}
}}

chance ruling: "The claim survives" by Court {{
  win "Survives" -> won p 1/2
  loss "Dismissed with prejudice" -> lost p 1/2
}}

terminal won: "Claim proceeds"
terminal lost: "Claim ends"
"#
    )
}

const CASE: &str = "case m\nuses dismissal ^0.1.0\nas_of 2026-10-26\nsummary \"s\"\n\n\
stakes {\n  dismissal::won: 10000.usd\n  dismissal::lost: 0.usd fee_shift 0.5\n}\n";

/// Checks the package under the strict profile, then solves the matter.
fn solve(name: &str, flagged: bool) -> serde_json::Value {
    let dir = scratch(name);
    let pkg = dir.join("lit/dismissal");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(
        pkg.join("Lit.toml"),
        "[package]\nname = \"dismissal\"\nversion = \"0.1.0\"\nlanguage = 1\npack_id = \"dismissal\"\n\
         forum = \"frcp\"\nauthority = \"FederalRule\"\ntitle = \"Dismissal\"\n\
         in_force = \"2009-12-01\"\nknown_at = \"2025-01-01\"\n\n\
         [package.roles]\nPlaintiff = \"self\"\nCourt = \"nature\"\n",
    )
    .unwrap();
    std::fs::write(pkg.join("main.lit"), package(flagged)).unwrap();
    std::fs::write(dir.join("cases/m.case"), CASE).unwrap();
    let check = litigate_in(&dir, &["check", "lit/dismissal", "--profile", "strict"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let run = litigate_in(&dir, &["run", "cases/m.case", "--op", "solve"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    run.json["result"].clone()
}

#[test]
fn flag_expansion_leaves_a_staked_solve_unchanged() {
    let plain = solve("flagged-stakes-plain", false);
    let flagged = solve("flagged-stakes-flagged", true);
    let value = |r: &serde_json::Value| r["result"]["value"].as_f64().unwrap();
    assert!(value(&plain) > 0.0, "{plain}");
    assert!(
        (value(&flagged) - value(&plain)).abs() < 1e-9,
        "flagged {} vs plain {}",
        value(&flagged),
        value(&plain)
    );
    assert!(
        !flagged.to_string().contains("payoff-not-authored"),
        "{flagged}"
    );
}
