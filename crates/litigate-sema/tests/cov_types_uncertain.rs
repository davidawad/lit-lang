//! Epistemic uncertainty in expressions (design review 2026-10-23 §2.11,
//! SPEC §6): `Credal` sets with interval arithmetic and explicit point
//! projections, and conditional distributions `Dist<E, C>`, with
//! diagnostics snapshotted.
mod common;
use common::check_body;

#[test]
fn credal_sets_fold_and_project() {
    let out = check_body(
        "default a when mean(credal(1/4, 3/4)) == 1/2 => 1\n\
         default b when lower(credal(0.2, 0.4)) == 1/5 && upper(credal(0.2, 0.4)) == 2/5 => 1\n\
         default c when true => credal(1/4, 1/2) + credal(1/8, 1/4)\n\
         default d when true => 1 - credal(1/4, 1/2)\n\
         default e when true => credal(1/4, 1/2) * (1/2)\n\
         default f when credal(1/4, 1/2) == credal(1/4, 1/2) => 1\n\
         default g when true => credal(?, 1/2)\n\
         default h when true => mean(g)\n\
         default i when true => g * g\n\
         type K = Credal",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn credal_sets_reject_what_is_not_a_set() {
    let out = check_body(
        "default a when true => credal(3/4, 1/4)\n\
         default b when true => credal(credal(0, 1), 1)\n\
         default c when true => mean(1/2)\n\
         default d when true => mean(nope)\n\
         default e when true => credal(1/2, 1) + credal(1/2, 1)\n\
         default f when true => credal(0, 1/4) - credal(1/2, 1)\n\
         default g when true => credal(?, 1/2)\n\
         default h when true => g + credal(0, 1/4)\n\
         default i when true => credal(0, 1) / credal(0, 1)\n\
         default j when credal(0, 1) < credal(0, 1) => 1\n\
         default k when true => mean(credal(1/9223372036854775807, 1/9223372036854775806))\n\
         default l when true => credal(1/9223372036854775807, 1) + credal(1/9223372036854775806, 1)\n\
         default m when true => lower()",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn a_credal_p_needs_a_projection() {
    let out = check_body(
        "chance n: \"N\" {\n  a \"A\" -> t p (credal(1/4, 1/2))\n  b \"B\" -> t p (mean(credal(1/4, 3/4)))\n}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn conditional_distributions_are_total_over_their_condition() {
    let out = check_body(
        "default a when prob(at({ complaint: { Personal: 1/4, Mail: 3/4 }, answer: { Personal: 1, Mail: 0 } }, Document.answer), Personal) == 1 => 1\n\
         default b when true => at({ complaint: { Personal: 1/4, Mail: 3/4 }, answer: { Personal: ?, Mail: 1/2 } }, Document.complaint)\n\
         default c when true => at({ complaint: { Personal: 1/4, Mail: 3/4 }, answer: { Personal: 1, Mail: 0 } }, ?)\n\
         event P(d: Dist<Method, Document>)\n\
         default d when has_event(P({ complaint: { Personal: 1/2, Mail: 1/2 }, answer: { Personal: 1/2, Mail: 1/2 } })) => 1\n\
         type D = Dist<Method, Document>",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn conditional_distributions_reject_bad_literals() {
    let out = check_body(
        "enum Other { Personal, Mail }\n\
         event P(d: Dist<Method, Document>)\n\
         default a when has_event(P({ complaint: { Personal: 1/2, Mail: 1/2 } })) => 1\n\
         default b when has_event(P({ complaint: { Personal: 1/2, Mail: 1/2 }, complaint: { Personal: 1, Mail: 0 }, answer: { Personal: 1, Mail: 0 }, reply: { Personal: 1, Mail: 0 } })) => 1\n\
         default c when has_event(P({ complaint: { Personal: 1/2 }, answer: { Personal: 1, Mail: 0 } })) => 1\n\
         default d when true => at({ nope: { Personal: 1 } }, x)\n\
         default e when true => at({ complaint: { Personal: 1, Mail: 0 }, answer: { Personal: 1, Mail: 0 } }, Document.answer)\n\
         default f when true => at({ complaint: 1, answer: 2 }, Document.answer)\n\
         default g when true => at({ complaint: { zip: 1 }, answer: { zip: 1 } }, Document.answer)\n\
         default h when true => at(1, Document.answer)\n\
         default i when true => at(nope, Document.answer)\n\
         type T1 = Dist<Method, Number>\n\
         type T2 = Dist<Method, Document, Method>",
    );
    insta::assert_debug_snapshot!(out);
}
