//! `.case` resolution (SPEC §3.2, §5.6): every resolver diagnostic, the
//! optional header items, and the recovery that keeps resolving the rest
//! of a matter after a syntax error.
mod cov_case_support;

use cov_case_support::{check, fixture, lock, matter, outcome, run, ALPHA_ONLY, STAKES};
use litigate_base::Profile;
use litigate_sema::case::NoClock;
use std::collections::BTreeMap;

const SERVED: &str = "2010-01-04 Served(complaint, Mail) on d";

fn header(head: &str) -> String {
    matter(ALPHA_ONLY, head, &[SERVED], STAKES)
}

#[test]
fn header_items_table() {
    let rows: Vec<(&str, String)> = vec![
        (
            "ok: known_at, notes, and sources with a date, a hole, and nothing",
            header("").replace(
                "summary \"s\"\n",
                "known_at 2025-06-01\nsummary \"s\"\nnotes \"n\"\n\
                 source \"A\" { url \"https://a.example\"\n as_of 2010-01-01 }\n\
                 source \"B\" { as_of ?(\"unknown\") }\nsource \"C\" { as_of ? }\nsource \"D\"\n",
            ),
        ),
        (
            "known_at: before the packages were known, nothing is in force",
            header("").replace("summary \"s\"\n", "known_at 2010-06-01\nsummary \"s\"\n"),
        ),
        (
            "known_at: not a valid date",
            header("").replace("summary \"s\"\n", "known_at 2010-02-30\nsummary \"s\"\n"),
        ),
        (
            "perspective: a role that is not self, opponent, or nature",
            header("perspective { Plaintiff: self, Court: boss }"),
        ),
        (
            "uses: a package that is not loaded",
            matter("alpha ^1.0.0, omega ^1.0.0", "", &[SERVED], STAKES),
        ),
        (
            "events: an event dated on no calendar day",
            matter(
                ALPHA_ONLY,
                "",
                &["2010-02-30 Served(complaint, Mail) on d"],
                STAKES,
            ),
        ),
        (
            "events: a line missing its party keeps the rest resolving",
            matter(
                ALPHA_ONLY,
                "",
                &[
                    "2010-01-04 Served(complaint, Mail)",
                    "2010-01-05 Nope() by d",
                ],
                STAKES,
            ),
        ),
        (
            "events: a cite hole is the authority as written",
            matter(
                ALPHA_ONLY,
                "",
                &["2010-01-04 Forfeited(twin-a) by d cite ?(\"standing order\")"],
                STAKES,
            ),
        ),
        (
            "events: an empty cite hole",
            matter(
                ALPHA_ONLY,
                "",
                &["2010-01-04 Forfeited(twin-a) by d cite ?"],
                STAKES,
            ),
        ),
        (
            "events: out of order, sorted by a machine-applicable fix",
            matter(
                ALPHA_ONLY,
                "",
                &["2010-01-05 Benched(complaint) by p", SERVED],
                STAKES,
            ),
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, text)| (*name, run(text, &NoClock, Profile::Dev)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn block_values_table() {
    let rows: Vec<(&str, &str)> = vec![
        (
            "stakes: more than two decimals is not money",
            "stakes { answered: 1.005.usd, defaulted: 0.usd }",
        ),
        (
            "stakes: an amount whose minor units overflow",
            "stakes { answered: 100000000000000000.usd, defaulted: 0.usd }",
        ),
        (
            "stakes: an amount beyond an exact rational",
            "stakes { answered: 99999999999999999999999.usd, defaulted: 0.usd }",
        ),
        (
            "stakes: an unterminated block still resolves its entries",
            "stakes { answered: 0.usd, defaulted: 0.usd",
        ),
        (
            "stakes: a multi-line block takes the missing stake on its own line",
            "stakes {\n  answered: 0.usd\n}",
        ),
        ("stakes: an empty block takes the missing stake after the brace", "stakes {}"),
        ("facts: a block with no record", "facts nothing\nstakes { answered: 0.usd, defaulted: 0.usd }"),
        (
            "beliefs: an observation with no node",
            "stakes { answered: 0.usd, defaulted: 0.usd }\nbeliefs { observe : { allow: 1 } }",
        ),
        (
            "beliefs: an outcome with no count",
            "stakes { answered: 0.usd, defaulted: 0.usd }\nbeliefs { observe bench: { allow: , deny: 1 } }",
        ),
        (
            "beliefs: a count that is not a non-negative integer",
            "stakes { answered: 0.usd, defaulted: 0.usd }\nbeliefs { observe bench: { allow: 1.5, deny: 1 } }",
        ),
        (
            "beliefs: an observation with no outcomes",
            "stakes { answered: 0.usd, defaulted: 0.usd }\nbeliefs { observe bench }",
        ),
        (
            "analysis: booleans, unset, nested records and lists are JSON",
            "stakes { answered: 0.usd, defaulted: 0.usd }\n\
             analysis { flag: true, off: false, gone: unset, params: { rate: 650 }, seeds: [1, 2] }",
        ),
        (
            "analysis: a number JSON cannot read, alone, nested, and listed",
            "stakes { answered: 0.usd, defaulted: 0.usd }\n\
             analysis { n: 007, nested: { m: 007 }, list: [007] }",
        ),
        (
            "analysis: a field with no value",
            "stakes { answered: 0.usd, defaulted: 0.usd }\nanalysis { rate: }",
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, tail)| {
            let text = matter(
                ALPHA_ONLY,
                "",
                &["2010-01-04 Benched(complaint) by p"],
                tail,
            );
            (*name, run(&text, &NoClock, Profile::Dev))
        })
        .collect();
    insta::assert_debug_snapshot!(table);
}

/// A matter with a syntax error resolves no `Case`; the resolver skips the
/// broken item and reports nothing beyond the parser.
#[test]
fn malformed_matters_table() {
    let item = |head: &str| header(head);
    let parties = |block: &str| {
        header("").replace(
            "parties {\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n  c: Court Tribunal\n}",
            block,
        )
    };
    let rows: Vec<(&str, String)> = vec![
        (
            "header: no case id",
            "case 5\nuses alpha ^1.0.0\nas_of 2011-06-01\nsummary \"s\"\n".into(),
        ),
        (
            "header: no as_of",
            "case t\nuses alpha ^1.0.0\nsummary \"s\"\n".into(),
        ),
        (
            "header: an as_of with no date",
            "case t\nuses alpha ^1.0.0\nas_of soon\nsummary \"s\"\n".into(),
        ),
        (
            "header: no summary",
            "case t\nuses alpha ^1.0.0\nas_of 2011-06-01\n".into(),
        ),
        (
            "header: no uses",
            "case t\nas_of 2011-06-01\nsummary \"s\"\n".into(),
        ),
        ("start: no node", item("start 5")),
        (
            "start: a deadline is not a node",
            item("start court-period"),
        ),
        (
            "source: no title",
            item("source { url \"https://a.example\" }"),
        ),
        (
            "source: an as_of neither a date nor a hole",
            item("source \"A\" { as_of 5 }"),
        ),
        (
            "perspective: neither a party nor roles",
            item("perspective 5"),
        ),
        (
            "perspective: a role with no actor",
            item("perspective { : self }"),
        ),
        (
            "perspective: an actor with no role",
            item("perspective { Plaintiff: }"),
        ),
        (
            "parties: a party with no entity type",
            parties("parties { p: Plaintiff, d: Defendant Corporation }"),
        ),
        (
            "parties: a party with no actor",
            parties("parties { p: , d: Defendant Corporation }"),
        ),
        (
            "stakes: entries with no terminal or no value",
            matter(
                ALPHA_ONLY,
                "",
                &[SERVED],
                "stakes { : 0.usd, answered: , defaulted: 0.usd }",
            ),
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, text)| (*name, run(text, &NoClock, Profile::Dev)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

/// A `uses` pin needs a `Lit.lock` entry that satisfies its requirement.
#[test]
fn pins_resolve_through_the_lock() {
    let text = matter(ALPHA_ONLY, "", &[SERVED], STAKES);
    let mut partial = lock(fixture());
    partial.packages.retain(|p| p.name != "alpha");
    let table: BTreeMap<&str, Vec<String>> = [
        ("no lock", None),
        ("a lock without the package", Some(&partial)),
    ]
    .into_iter()
    .map(|(name, lock)| {
        let checked = check(&text, fixture(), lock, &NoClock, Profile::Dev);
        assert!(checked.case.is_none());
        (name, outcome(&checked))
    })
    .collect();
    insta::assert_debug_snapshot!(table);
}
