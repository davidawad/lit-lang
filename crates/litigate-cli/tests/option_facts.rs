//! Option-scoped facts (lg-teqz): a decision option sets a flag that a
//! later chance node's distribution is conditioned on (`requires`/`forbids`
//! on its edges, `Dist[E | C]`), so options that reach the same ruling
//! node no longer reach the same odds. The synthetic model is a 28 U.S.C.
//! § 636(c) magistrate-judge consent decision: consent makes the
//! magistrate judge's ruling a final order, declining sends the matter to
//! a district judge, and silence leaves the ruling exposed as void for
//! want of consent (Williams v. King, 875 F.3d 500 (9th Cir. 2017)). The
//! numbers are illustrative, not calibrated.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{litigate_in, scratch};

/// The consent package: `ruling` is conditioned on the consent flags
/// when `gated`, the same draw for every option otherwise.
fn package(gated: bool) -> String {
    let ruling = if gated {
        r#"  final-win "Granted (final order)" -> won p 1/2 { requires [consented] }
  final-loss "Denied (final order)" -> lost p 1/2 { requires [consented] }
  district-win "Granted (district)" -> won p 7/10 { requires [declined], forbids [consented] }
  district-loss "Denied (district)" -> lost p 3/10 { requires [declined], forbids [consented] }
  void-win "Granted, open to vacatur" -> won p 1/10 { forbids [consented, declined] }
  void-loss "Denied or vacated" -> lost p 9/10 { forbids [consented, declined] }"#
    } else {
        r#"  win "Granted" -> won p 1/2
  loss "Denied" -> lost p 1/2"#
    };
    format!(
        r#"module demo.consent

package consent {{ forum frcp, authority FederalRule }}

start consent

decision consent: "Consent to a magistrate judge" by Plaintiff {{
  consent "Consent" -> ruling {{ sets [consented] }}
  decline "Decline" -> ruling {{ cost 500.usd, sets [declined] }}
  silence "Say nothing" -> ruling
}}

chance ruling: "Ruling on the Rule 60(b) motion" by Court {{
{ruling}
}}

terminal won: "Relief granted" {{ payoff 10000.usd }}
terminal lost: "Relief denied" {{ payoff 0.usd }}
"#
    )
}

/// The solved best line's first edge id and value for the consent model.
fn solve(name: &str, gated: bool) -> (String, f64) {
    let dir = scratch(name);
    let pkg = dir.join("lit/consent");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(
        pkg.join("Lit.toml"),
        "[package]\nname = \"consent\"\nversion = \"0.1.0\"\nlanguage = 1\npack_id = \"consent\"\n\
         forum = \"frcp\"\nauthority = \"FederalRule\"\ntitle = \"Consent\"\n\
         in_force = \"2009-12-01\"\nknown_at = \"2025-01-01\"\n\n\
         [package.roles]\nPlaintiff = \"self\"\nCourt = \"nature\"\n",
    )
    .unwrap();
    std::fs::write(pkg.join("main.lit"), package(gated)).unwrap();
    std::fs::write(
        dir.join("cases/m.case"),
        "case m\nuses consent ^0.1.0\nas_of 2026-06-01\nsummary \"s\"\n",
    )
    .unwrap();
    let check = litigate_in(&dir, &["check", "lit/consent", "--profile", "strict"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let run = litigate_in(&dir, &["run", "cases/m.case", "--op", "solve"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"]["result"];
    (
        result["best_line"][0]["id"].as_str().unwrap().to_owned(),
        result["value"].as_f64().unwrap(),
    )
}

#[test]
fn an_option_that_changes_the_odds_downstream_is_preferred() {
    assert_eq!(
        solve("option-facts-gated", true),
        ("consent::decline".to_owned(), 6500.0)
    );
}

#[test]
fn options_reaching_one_distribution_pick_the_cheapest() {
    let (edge, value) = solve("option-facts-ungated", false);
    assert_ne!(edge, "consent::decline");
    assert!((value - 5000.0).abs() < 1e-9, "{value}");
}
