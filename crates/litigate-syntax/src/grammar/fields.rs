//! Keyword fields (`keyword value`) of node bodies, edges, deadline
//! options, counting rules, instances, packages, and sources.
use super::clauses::keyword_field;
use super::values::{
    boolean, call, cite, date, date_or_hole, duration, duration_estimate, id_list, interval,
    money_or_hole, number, prob_value, record, reference, replaces_list, require, signed_money,
    signed_number, string, string_or_hole, Key, Value,
};
use super::{brace_list, delimited_node, Sep};
use crate::error::{range, SyntaxError};
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::DiagnosticCode;

/// A field keyword and its value parser (`None`: the keyword stands alone).
pub(crate) type Spec = (SyntaxKind, Option<Value>);

pub(crate) const NODE_FIELDS: &[Spec] = &[
    (KwGroup, Some(reference)),
    (KwNote, Some(string)),
    (KwValence, Some(reference)),
    (KwCourtListenerUrl, Some(string)),
    (KwTags, Some(id_list)),
    (KwAttrs, Some(num_record)),
    (KwPayoff, Some(money_or_hole)),
    (KwPayoffByFlag, Some(money_record)),
    (KwFeeShift, Some(number)),
    (KwBasis, Some(bound)),
    (KwFormerly, Some(id_list)),
];

pub(crate) const EDGE_FIELDS: &[Spec] = &[
    (KwEvent, Some(call)),
    (KwAuthority, Some(string_or_cite)),
    (KwEffort, Some(duration)),
    (KwCost, Some(money_or_hole)),
    (KwWithin, Some(within)),
    (KwDeadline, Some(deadline_literal)),
    (KwDuration, Some(duration_value)),
    (KwValence, Some(reference)),
    (KwCourtListenerUrl, Some(string)),
    (KwActionId, Some(string)),
    (KwNote, Some(string)),
    (KwTags, Some(id_list)),
    (KwAttrs, Some(num_record)),
    (KwSets, Some(id_list)),
    (KwClears, Some(id_list)),
    (KwRequires, Some(id_list)),
    (KwForbids, Some(id_list)),
    (KwSpawn, Some(id_list)),
    (KwJoins, Some(id_list)),
    (KwAnytime, None),
    (KwReplaces, Some(replaces_list)),
    (KwWhen, Some(bound)),
    (KwBasis, Some(bound)),
    (KwFormerly, Some(id_list)),
];

pub(crate) const DEADLINE_OPTS: &[Spec] = &[
    (KwExtendable, Some(boolean)),
    (KwExtensionAuthority, Some(string_or_hole)),
    (KwNote, Some(string)),
    (KwCounting, Some(string)),
];

/// Options of a `deadline` declaration: the [`DEADLINE_OPTS`] of a
/// `within` edge, plus the obligation it creates (SPEC §5.3): who bears
/// it, how a repeated trigger is treated, the capabilities that extend
/// or waive it, the trigger parameter that designates its due date, the
/// interpretation its least time rests on, and the cited terms its period
/// is derived from.
pub(crate) const DEADLINE_DECL_OPTS: &[Spec] = &[
    (KwExtendable, Some(boolean)),
    (KwExtensionAuthority, Some(string_or_hole)),
    (KwNote, Some(string)),
    (KwBearer, Some(ref_or_call)),
    (KwTrigger, Some(ref_or_call)),
    (KwExtend, Some(capability)),
    (KwWaive, Some(capability)),
    (KwDesignated, Some(reference)),
    (KwForum, Some(adopted_forum)),
    (KwInterpretation, Some(string)),
    (KwCharacterization, Some(characterization)),
    (KwLock, Some(lock_against)),
    (KwDerived, Some(derived_term)),
];

/// Fields of a `delegable` capability (besides `cite`): its principal,
/// operations, bound, condition and procedure.
pub(crate) const DELEGATION_FIELDS: &[Spec] = &[
    (KwTo, Some(ref_or_list)),
    (KwOp, Some(ref_or_list)),
    (KwMax, Some(duration)),
    (KwDiscretion, Some(reference)),
    (KwOn, Some(call)),
];

/// Fields of a `grant` body.
pub(crate) const GRANT_FIELDS: &[Spec] = &[
    (KwUnlock, None),
    (KwExtend, Some(capability)),
    (KwWaive, Some(capability)),
    (KwDelegable, Some(delegation)),
];

/// Fields of an `extend`/`waive` capability.
pub(crate) const CAPABILITY_FIELDS: &[Spec] = &[
    (KwBy, Some(reference)),
    (KwMax, Some(duration)),
    (KwTo, Some(bound)),
    (KwDiscretion, Some(reference)),
    (KwAfterExpiry, None),
    (KwOn, Some(call)),
    (KwWithin, Some(bound)),
    (KwUnless, Some(call)),
    (KwNever, Some(cite)),
    (KwInterpretation, Some(string)),
];

/// The effect clause of an `event` declaration (SPEC §5.2).
pub(crate) const EVENT_EFFECTS: &[Spec] = &[
    (KwRemoves, Some(reference)),
    (KwElects, Some(reference)),
    (KwExtends, Some(reference)),
    (KwTo, Some(reference)),
    (KwWaives, Some(reference)),
    (KwPaper, Some(reference)),
];

pub(crate) const COUNTING_ITEMS: &[Spec] = &[
    (KwExcludeTriggerDay, None),
    (KwIncludeLastDay, None),
    (KwAllowHours, None),
    (KwEndOfMonth, Some(reference)),
    (KwRollForwardOn, Some(id_list)),
    (KwServiceExtension, Some(service_extension)),
    (KwHolidays, Some(reference)),
    (KwForwardHolidays, Some(cited_reference)),
    (KwDeclaredHolidays, Some(cited_reference)),
    (KwForums, Some(id_list)),
    (KwAnchor, Some(reference)),
    (KwShortPeriod, Some(short_period)),
];

pub(crate) const INSTANCE_FIELDS: &[Spec] = &[
    (KwPack, Some(reference)),
    (KwNote, Some(string)),
    (KwRemoveEdges, Some(id_list)),
    (KwProbabilities, Some(probabilities)),
    (KwRoles, Some(name_record)),
    (KwPayoffTransform, Some(super::expr::paren_expr)),
    (KwStake, Some(money_or_hole)),
];

/// Items of a `calibration` body: its description, the sources its values
/// come from, and one entry per calibrated value.
pub(crate) const CALIBRATION_ITEMS: &[Spec] = &[
    (KwNote, Some(string)),
    (KwSource, Some(calibration_source)),
    (KwP, Some(calibrated_probability)),
    (KwDuration, Some(calibrated_duration)),
    (KwAttr, Some(calibrated_attr)),
];

/// Fields of a calibration `source`.
const CALIBRATION_SOURCE_FIELDS: &[Spec] = &[
    (KwUrl, Some(string)),
    (KwVintage, Some(interval)),
    (KwRetrieved, Some(date)),
];

/// Fields of one calibrated value.
const CALIBRATED_FIELDS: &[Spec] = &[
    (KwSource, Some(reference)),
    (KwN, Some(number)),
    (KwMethod, Some(string)),
];

pub(crate) const SOURCE_FIELDS: &[Spec] = &[(KwUrl, Some(string)), (KwAsOf, Some(date_or_hole))];

/// Keywords of a spec table (for "expected" messages).
pub(crate) fn keywords(specs: &[Spec]) -> Vec<SyntaxKind> {
    specs.iter().map(|(kw, _)| *kw).collect()
}

/// Parses one field from `specs`, or returns false untouched.
pub(crate) fn field(p: &mut Parser<'_>, specs: &[Spec]) -> bool {
    let Some((kw, value)) = specs.iter().find(|(kw, _)| p.at_kw(*kw)) else {
        return false;
    };
    keyword_field(p, *kw, *value);
    true
}

/// `{ field Sep }` over `specs` as a `FieldList`.
pub(crate) fn field_list(p: &mut Parser<'_>, specs: &[Spec]) -> bool {
    if !p.at(LBrace) {
        return false;
    }
    let expected = keywords(specs);
    brace_list(p, Sep::Optional, &expected, &mut |p| field(p, specs));
    true
}

/// Braced fields that must appear exactly once each, in order. As a
/// `FieldList` of `Field`s, or (`as_record`) a `RecordExpr` of
/// `key: value` `RecordField`s.
pub(crate) fn fixed_fields(p: &mut Parser<'_>, specs: &[Spec], as_record: bool) -> bool {
    ordered_fields(p, specs, as_record, &[])
}

/// [`fixed_fields`] where the keywords in `optional` may be left out.
fn ordered_fields(
    p: &mut Parser<'_>,
    specs: &[Spec],
    as_record: bool,
    optional: &[SyntaxKind],
) -> bool {
    if !p.at(LBrace) {
        return false;
    }
    let mut index = 0usize;
    let mut elem = |p: &mut Parser<'_>| {
        while specs
            .get(index)
            .is_some_and(|(kw, _)| optional.contains(kw) && !p.at_kw(*kw))
        {
            index += 1;
        }
        let Some((kw, value)) = specs.get(index) else {
            return false;
        };
        index += 1;
        if !p.at_kw(*kw) {
            p.error_expected(&[*kw]);
            p.skip_to(&[Comma, RBrace]);
            return true;
        }
        if as_record {
            p.start(RecordField);
            p.bump_as(*kw);
            if p.expect(Colon) {
                value.map(|value| require(p, value, &[Ident]));
            }
            p.finish();
        } else {
            keyword_field(p, *kw, *value);
        }
        true
    };
    let kind = if as_record { RecordExpr } else { FieldList };
    delimited_node(
        p,
        kind,
        (LBrace, RBrace),
        Sep::Optional,
        &keywords(specs),
        &mut elem,
    );
    let missing = specs
        .get(index..)
        .unwrap_or_default()
        .iter()
        .find(|(kw, _)| !optional.contains(kw));
    if let Some((missing, _)) = missing {
        let end = p.last_end();
        let text = missing.text().unwrap_or_default();
        p.push_error(SyntaxError::new(
            DiagnosticCode::UnexpectedToken,
            range(end.saturating_sub(1), end),
            format!("expected `{text}` before the closing `}}`"),
        ));
    }
    true
}

fn string_or_cite(p: &mut Parser<'_>) -> bool {
    string(p) || cite(p)
}

fn num_record(p: &mut Parser<'_>) -> bool {
    record(p, Key::Ident, signed_number)
}

fn money_record(p: &mut Parser<'_>) -> bool {
    record(p, Key::Ident, signed_money)
}

fn name_record(p: &mut Parser<'_>) -> bool {
    record(p, Key::Ident, reference)
}

fn probabilities(p: &mut Parser<'_>) -> bool {
    record(p, Key::Ident, prob_value)
}

/// A date bound of a capability: an expression such as
/// `max(prescribed_due + 30.days, grant_order_entered + 14.days)`.
fn bound(p: &mut Parser<'_>) -> bool {
    super::expr::expr(p)
}

/// `{ by Actor, max Duration | to Bound, discretion Standard,
/// after_expiry, on Motion(), within Bound, unless Denial() }` or `{ never
/// cite "X" }`.
fn capability(p: &mut Parser<'_>) -> bool {
    field_list(p, CAPABILITY_FIELDS)
}

/// `{ to Level | [Level, ...], op Op | [Op, ...], max Duration, discretion
/// Standard, on Motion(), cite X }`.
pub(crate) fn delegation(p: &mut Parser<'_>) -> bool {
    if !p.at(LBrace) {
        return false;
    }
    let mut expected = keywords(DELEGATION_FIELDS);
    expected.push(KwCite);
    brace_list(p, Sep::Optional, &expected, &mut |p| {
        field(p, DELEGATION_FIELDS) || cite(p)
    });
    true
}

/// `characterization (jurisdictional | claim_processing) [Cite]`.
fn characterization(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        cite(p);
        true
    }
}

/// `against [L, ...] [cite "X"]`, the value of a named-authority lock
/// (`lock against [LocalRule, CaseOrder] cite "FRAP 26(b)(1)"`); sema
/// requires the cite.
fn lock_against(p: &mut Parser<'_>) -> bool {
    if !p.expect_kw(KwAgainst) {
        return true;
    }
    if require(p, id_list, &[LBracket]) {
        cite(p);
    }
    true
}

/// A name or a bracketed list of names.
fn ref_or_list(p: &mut Parser<'_>) -> bool {
    reference(p) || id_list(p)
}

/// A policy name (`first`) or a call (`later_of(other)`).
fn ref_or_call(p: &mut Parser<'_>) -> bool {
    if !p.at(Ident) {
        return false;
    }
    let checkpoint = p.checkpoint();
    reference(p);
    if p.at(LParen) {
        p.start_at(checkpoint, CallExpr);
        super::expr::arg_list(p);
        p.finish();
    }
    true
}

fn deadline_opts(p: &mut Parser<'_>) {
    field_list(p, DEADLINE_OPTS);
}

/// `within Ref [DeadlineOpts]`.
fn within(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        deadline_opts(p);
        true
    }
}

/// `deadline Duration [DeadlineOpts]`.
fn deadline_literal(p: &mut Parser<'_>) -> bool {
    duration(p) && {
        deadline_opts(p);
        true
    }
}

/// `duration (Duration | { [min D] mode D [max D] })`. These are
/// elapsed-time estimates, so day amounts may be decimal.
fn duration_value(p: &mut Parser<'_>) -> bool {
    const RANGE: &[Spec] = &[
        (KwMin, Some(duration_estimate)),
        (KwMode, Some(duration_estimate)),
        (KwMax, Some(duration_estimate)),
    ];
    duration_estimate(p) || ordered_fields(p, RANGE, false, &[KwMin, KwMax])
}

/// `Ident String { url String, vintage Interval, retrieved Date }`.
fn calibration_source(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        if require(p, string, &[String]) {
            require(p, calibration_source_fields, &[LBrace]);
        }
        true
    }
}

fn calibration_source_fields(p: &mut Parser<'_>) -> bool {
    field_list(p, CALIBRATION_SOURCE_FIELDS)
}

fn calibrated_fields(p: &mut Parser<'_>) -> bool {
    field_list(p, CALIBRATED_FIELDS)
}

/// `Ref ProbValue { source Ident, n Number, method String }`: a hole
/// clears the edge's probability.
fn calibrated_probability(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        if require(p, prob_value, &[Number, Question]) {
            require(p, calibrated_fields, &[LBrace]);
        }
        true
    }
}

/// `Ref (Duration | { [min D] mode D [max D] }) { ... }`.
fn calibrated_duration(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        if require(p, duration_value, &[Duration, LBrace]) {
            require(p, calibrated_fields, &[LBrace]);
        }
        true
    }
}

/// `Ref Ident SignedNumber { ... }`: one numeric attribute of a node.
fn calibrated_attr(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        if require(p, reference, &[Ident]) && require(p, signed_number, &[Number]) {
            require(p, calibrated_fields, &[LBrace]);
        }
        true
    }
}

/// `service_extension { Method: Duration | unrecognized } [Cite]`.
fn service_extension(p: &mut Parser<'_>) -> bool {
    record(p, Key::Ident, service_time) && {
        cite(p);
        true
    }
}

/// What one service method adds: `N.days`, or `unrecognized` (a reference
/// sema checks).
fn service_time(p: &mut Parser<'_>) -> bool {
    duration(p) || reference(p)
}

/// `Ref [cite ... [quote "..."]]`: the forum whose rules set a period
/// and the rule adopting them for it, with the words that adopt them
/// (SPEC §5.3).
fn adopted_forum(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        if cite(p) {
            quote(p);
        }
        true
    }
}

/// `Duration cite "X" [quote "..."]`: one term of a deadline's period
/// derived from several rules, and the rule stating it.
fn derived_term(p: &mut Parser<'_>) -> bool {
    duration(p) && {
        if require(p, cite, &[KwCite]) {
            quote(p);
        }
        true
    }
}

/// `[quote "..."]` after an adopting or a derived term's cite.
fn quote(p: &mut Parser<'_>) {
    if p.at_kw(KwQuote) {
        p.bump_as(KwQuote);
        require(p, string, &[String]);
    }
}

/// `Ref [cite ...]`: a calendar and the rule making it one the forum
/// closes on (`forward_holidays`, `declared_holidays`).
fn cited_reference(p: &mut Parser<'_>) -> bool {
    reference(p) && {
        cite(p);
        true
    }
}

/// `short_period { below: Duration, count: Ident }`.
fn short_period(p: &mut Parser<'_>) -> bool {
    const KEYS: &[Spec] = &[(KwBelow, Some(duration)), (KwCount, Some(reference))];
    fixed_fields(p, KEYS, true)
}

/// `priority Number` and other number-valued fields.
pub(crate) fn number_value(p: &mut Parser<'_>) -> bool {
    number(p)
}

/// Value of a stake: signed money or `unvalued`, then an optional
/// `fee_shift Number` field; or the field alone.
pub(crate) fn stake_value(p: &mut Parser<'_>) -> bool {
    let valued = signed_money(p) || super::values::keyword_literal(p, &[KwUnvalued]);
    let shifted = p.at_kw(KwFeeShift);
    if shifted {
        p.start(Field);
        p.bump_as(KwFeeShift);
        require(p, number, &[Number]);
        p.finish();
    }
    valued || shifted
}

/// `patch Ident PatchFields` inside an instance: typed edge fields, each
/// replacing the base edge's, and [`guard_op`]s.
pub(crate) fn patch(p: &mut Parser<'_>) -> bool {
    if !p.at_kw(KwPatch) {
        return false;
    }
    p.start(PatchDecl);
    p.bump_as(KwPatch);
    if p.expect(Ident) && !patch_fields(p) {
        p.error_expected(&[LBrace]);
    }
    p.finish();
    true
}

/// `PatchFields = "{" { (EdgeField | GuardOp) Sep } "}"` as a `FieldList`.
fn patch_fields(p: &mut Parser<'_>) -> bool {
    if !p.at(LBrace) {
        return false;
    }
    let mut expected = keywords(EDGE_FIELDS);
    expected.push(KwSupersedes);
    brace_list(p, Sep::Optional, &expected, &mut |p| {
        guard_op(p) || field(p, EDGE_FIELDS)
    });
    true
}

/// `GuardOp = ("when" ("and" | "or") | "supersedes" "when") Expr
/// { "by" Ident | VersionClause | Cite }` (SPEC §3.6): a typed operation
/// on the patched edge's guard (narrow, widen, replace) with its
/// authority. `and` and `or` are keywords only here; the lexer reports
/// them anywhere else ([`crate::GUARD_OP_WORDS`]).
fn guard_op(p: &mut Parser<'_>) -> bool {
    let combines = p.at_kw(KwWhen) && (p.nth_kw(1, KwAnd) || p.nth_kw(1, KwOr));
    if !combines && !p.at_kw(KwSupersedes) {
        return false;
    }
    p.start(GuardOp);
    if combines {
        p.bump_as(KwWhen);
        let op = if p.at_kw(KwAnd) { KwAnd } else { KwOr };
        p.bump_as(op);
    } else {
        p.bump_as(KwSupersedes);
    }
    if combines || p.at_kw(KwWhen) {
        if !combines {
            p.bump_as(KwWhen);
        }
        super::expr::require_expr(p);
    } else {
        p.error_expected(&[KwWhen]);
    }
    loop {
        if p.at_kw(KwBy) && p.nth(1) == Ident {
            keyword_field(p, KwBy, Some(reference));
        } else if !(super::clauses::version_clause(p) || super::values::cite(p)) {
            break;
        }
    }
    p.finish();
    true
}
