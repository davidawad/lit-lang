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
    let result = solve_package(name, &package(gated));
    (
        result["best_line"][0]["id"].as_str().unwrap().to_owned(),
        result["value"].as_f64().unwrap(),
    )
}

/// Checks `main.lit` under the strict profile, then solves a matter on it.
fn solve_package(name: &str, main: &str) -> serde_json::Value {
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
    std::fs::write(pkg.join("main.lit"), main).unwrap();
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
    run.json["result"]["result"].clone()
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

/// The forum follows the option too, past a node that reads no flag:
/// declining consent reassigns the matter to a district judge, a
/// consented or silent matter stays with the magistrate judge, and there
/// the ruling's odds still turn on whether consent was given. Silence can
/// be cured later by conduct (implied consent, Roell v. Withrow, 538 U.S.
/// 580 (2003)), at a price.
const FORUM: &str = r#"module demo.forum

package consent { forum frcp, authority FederalRule }

start consent

decision consent: "Consent to a magistrate judge" by Plaintiff {
  consent "Consent" -> briefing { cost 100.usd, sets [consented] }
  decline "Decline" -> briefing { cost 500.usd, sets [declined] }
  silence "Say nothing" -> briefing
}

chance briefing: "Motion fully briefed" by Court { briefed "Briefed" -> assign p 1 }

chance assign: "Who decides the motion" by Court {
  magistrate "Stays with the magistrate judge" -> conduct p 1 { forbids [declined] }
  reassigned "Reassigned to a district judge" -> district p 1 { requires [declined] }
}

decision conduct: "Litigate before the magistrate judge" by Plaintiff {
  appear "Appear and argue without objection" -> magistrate { cost 300.usd, sets [consented] }
  stand "Stand on the papers" -> magistrate
}

chance magistrate: "Magistrate judge rules" by Court {
  final-win "Granted (final order)" -> won p 2/5 { requires [consented] }
  final-loss "Denied (final order)" -> lost p 3/5 { requires [consented] }
  void-win "Granted, open to vacatur" -> won p 1/10 { forbids [consented] }
  void-loss "Denied or vacated" -> lost p 9/10 { forbids [consented] }
}

chance district: "District judge rules" by Court {
  win "Granted" -> won p 7/10
  loss "Denied" -> lost p 3/10
}

terminal won: "Relief granted" { payoff 10000.usd }
terminal lost: "Relief denied" { payoff 0.usd }
"#;

/// The edge ids of a solved best line.
fn line(result: &serde_json::Value) -> Vec<&str> {
    result["best_line"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect()
}

#[test]
fn an_option_that_changes_the_forum_downstream_is_preferred() {
    let result = solve_package("option-facts-forum", FORUM);
    assert_eq!(result["value"].as_f64(), Some(6500.0));
    let line = line(&result);
    assert_eq!(line.first(), Some(&"consent::decline"), "{line:?}");
    assert!(line.contains(&"consent::reassigned{declined}"), "{line:?}");
}

#[test]
fn silence_cured_by_conduct_reaches_the_consented_odds() {
    // Express consent now costs more than appearing later: silence then
    // appearance (4000 - 300) beats consent (4000 - 1000), declining
    // (7000 - 5000) and silence standing on the papers (1000).
    let main = FORUM
        .replace(
            "cost 100.usd, sets [consented]",
            "cost 1000.usd, sets [consented]",
        )
        .replace(
            "cost 500.usd, sets [declined]",
            "cost 5000.usd, sets [declined]",
        );
    let result = solve_package("option-facts-implied", &main);
    assert_eq!(result["value"].as_f64(), Some(3700.0));
    let line = line(&result);
    assert_eq!(line.first(), Some(&"consent::silence"), "{line:?}");
    assert!(line.contains(&"consent::appear"), "{line:?}");
}
