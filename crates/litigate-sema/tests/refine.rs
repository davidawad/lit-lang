//! Ground refinements (SPEC §6, §13.1; design review §2.2 tier 1): a
//! `Probability` term over unknowns is decided exactly; every rejected
//! term is an L2006 type error whose note names the refinement and a
//! ground counterexample.
mod common;
use common::{package, workspace, PRELUDE};

/// `line:col code message | notes` for each diagnostic of a one-module
/// package whose body is `body`.
fn diagnostics(body: &str) -> Vec<String> {
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .map(|d| {
            let notes: Vec<&str> = d.notes.iter().map(|n| n.message.as_str()).collect();
            format!(
                "{}:{} {} {} | {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message,
                notes.join(" | ")
            )
        })
        .collect()
}

/// Two `Probability` defaults: references to them do not fold, so each is
/// an atom in `[0, 1]` (an exception may rebut either with any probability).
const ATOMS: &str = "default a when true => prob({ Personal: 1/4, Mail: 3/4 }, Mail)
default b when true => prob({ Personal: 1/4, Mail: 3/4 }, Personal)
";

fn check(defaults: &str) -> Vec<String> {
    diagnostics(&format!("{ATOMS}{defaults}"))
}

#[test]
fn probability_arithmetic_over_unknowns_is_decided_exactly() {
    // T1 (design review §5.3): `a + b` was accepted with a = 0.7, b = 0.6.
    assert_eq!(
        check("default c when true => a + b"),
        ["12:24 L2006 probability `a + b` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when a = 1, b = 1: `a + b` = 2 > 1"]
    );
    assert_eq!(
        check("default c when true => a - b"),
        ["12:24 L2006 probability `a - b` is not provably in [0, 1]: it ranges over [-1, 1] | refinement `Probability = {q | 0 <= q <= 1}` fails when a = 0, b = 1: `a - b` = -1 < 0"]
    );
    // Scaling by a constant stays linear: 2a is not a probability.
    assert_eq!(
        check("default c when true => a / (1/2)"),
        ["12:24 L2006 probability `2 * a` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when a = 1: `2 * a` = 2 > 1"]
    );
    assert_eq!(
        check("default c when true => 1/2 - a"),
        ["12:24 L2006 probability `1/2 - a` is not provably in [0, 1]: it ranges over [-1/2, 1/2] | refinement `Probability = {q | 0 <= q <= 1}` fails when a = 1: `1/2 - a` = -1/2 < 0"]
    );
}

#[test]
fn terms_that_stay_in_the_unit_interval_are_accepted() {
    // The complement, a convex combination, an exact cancellation (one
    // name is one unknown), a product of probabilities, and a quotient
    // whose divisor is bounded away from 0.
    assert_eq!(
        check(
            "default c when true => 1 - a
default d when true => a * (1/2) + b * (1/2)
default e when true => (1 - a) + a
default f when true => a * b
default g when true => (a * (1/2)) / (1/2 + b * (1/2))
default h when true => 1/4 * a + 3/4 * (1 - b)
default i when true => a - a"
        ),
        Vec::<String>::new()
    );
}

#[test]
fn every_probability_subterm_is_itself_checked() {
    // `(a + b) / 2` is in [0, 1], but its subterm `a + b` is not: each
    // `Probability` subterm has the refinement, as constants do.
    assert_eq!(
        check("default c when true => (a + b) / 2"),
        ["12:25 L2006 probability `a + b` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when a = 1, b = 1: `a + b` = 2 > 1"]
    );
}

#[test]
fn nonlinear_terms_are_boxed_by_interval_arithmetic() {
    // `a * b` leaves the linear fragment: a fresh atom boxed in [0, 1].
    // Twice it is [0, 2].
    assert_eq!(
        check("default c when true => a * b + a * b"),
        ["12:24 L2006 probability `2 * (a) * (b)` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when (a) * (b) = 1: `2 * (a) * (b)` = 2 > 1"]
    );
    // A quotient over a divisor in [1/2, 1] is boxed by the corners.
    assert_eq!(
        check("default c when true => a / (1/2 + b * (1/2))"),
        ["12:24 L2006 probability `(a) / (1/2 + 1/2 * b)` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when (a) / (1/2 + 1/2 * b) = 2: `(a) / (1/2 + 1/2 * b)` = 2 > 1"]
    );
}

#[test]
fn a_divisor_that_may_be_zero_is_rejected() {
    assert_eq!(
        check("default c when true => a / b"),
        ["12:24 L2006 divisor `b` may be 0: it ranges over [0, 1] | refinement `{d | d != 0}` on a divisor fails; divide by a constant, or by a term bounded away from 0"]
    );
    assert_eq!(
        check("default c when true => b / (a - a)"),
        ["12:24 L2006 division by zero | "]
    );
    assert_eq!(
        check("default c when true => a / 0"),
        ["12:24 L2006 division by zero | "]
    );
}

#[test]
fn calls_and_matches_over_unknowns_are_atoms() {
    // `prob` of an unknown variant, and a `match` on one, do not fold:
    // each is an atom in [0, 1], so twice one is not a probability.
    assert_eq!(
        check(
            "default m when true => Method.Mail
default c when true => prob({ Personal: 1/4, Mail: 3/4 }, m) + prob({ Personal: 1/4, Mail: 3/4 }, m)
default d when true => 1 - (match m { Personal => a, Mail => b })
default e when true => (match m { Personal => a, Mail => b }) + (match m { Personal => a, Mail => b })"
        ),
        [
            "13:24 L2006 probability `2 * prob(<expr>, m)` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when prob(<expr>, m) = 1: `2 * prob(<expr>, m)` = 2 > 1",
            "15:24 L2006 probability `<expr> + <expr>` is not provably in [0, 1]: it ranges over [0, 2] | refinement `Probability = {q | 0 <= q <= 1}` fails when <expr> = 1, <expr> = 1: `<expr> + <expr>` = 2 > 1",
        ]
    );
}

#[test]
fn a_self_edge_does_not_hide_the_distribution_of_a_chance_node() {
    // T4 (design review §5.3): the self edge's absent `p` read as a hole,
    // so the exact-sum check was skipped.
    assert_eq!(
        diagnostics(
            "chance wait: \"W\" { a \"A\" -> wait, b \"B\" -> t p 1/2 }
chance fine: \"F\" { a \"A\" -> fine, b \"B\" -> t p 1/2, c \"C\" -> s p 1/2 }
chance spin: \"S\" { a \"A\" -> spin }"
        ),
        [
            "10:8 L2003 probabilities of chance node `wait` sum to 1/2, not exactly 1 | refinement `Dist<Edges(n)> = {m | Σ m(e) = 1}` over the edges other than self edges",
            "12:8 L2003 probabilities of chance node `spin` sum to 0, not exactly 1 | refinement `Dist<Edges(n)> = {m | Σ m(e) = 1}` over the edges other than self edges",
        ]
    );
}

#[test]
fn only_chance_nodes_carry_probabilities() {
    // Design review §6 item 11: a node is a choice or a draw, never both,
    // so `p` on a state's or a decision's edge is L2008 and no mixed
    // distribution is summed.
    assert_eq!(
        diagnostics(
            "state all: \"A\" { a \"A\" -> t p 1/2, b \"B\" -> s p 1/4 }
decision over: \"O\" { a \"A\" -> t p 3/4, c \"C\" -> t }
state none: \"N\" { a \"A\" -> t, b \"B\" -> none }"
        ),
        [
            "10:31 L2008 `p` belongs only to the edges of a `chance` node | a node is a choice or a draw, never both: make the node `chance`, or drop `p` and leave the choice to its actor",
            "10:49 L2008 `p` belongs only to the edges of a `chance` node | a node is a choice or a draw, never both: make the node `chance`, or drop `p` and leave the choice to its actor",
            "11:35 L2008 `p` belongs only to the edges of a `chance` node | a node is a choice or a draw, never both: make the node `chance`, or drop `p` and leave the choice to its actor",
        ]
    );
}

#[test]
fn a_bound_that_overflows_exact_arithmetic_is_an_error() {
    // The upper bound 1/p + 1/q + 1/3 over two primes near 2^32 has a
    // denominator past u64: exact arithmetic overflows, and says so.
    assert_eq!(
        check("default c when true => a * (1/4294967291) + b * (1/4294967279) + (a * b) * (1/3)"),
        ["12:24 L2006 exact arithmetic overflowed | "]
    );
    // A leading negative term renders with its sign.
    assert_eq!(
        check("default c when true => 0 - a"),
        ["12:24 L2006 probability `-a` is not provably in [0, 1]: it ranges over [-1, 0] | refinement `Probability = {q | 0 <= q <= 1}` fails when a = 1: `-a` = -1 < 0"]
    );
}
