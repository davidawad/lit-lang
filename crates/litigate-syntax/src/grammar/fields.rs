//! Keyword fields (`keyword value`) of node bodies, edges, deadline
//! options, counting rules, instances, packages, and sources.
use super::clauses::keyword_field;
use super::values::{
    boolean, call, cite, date_or_hole, duration, duration_estimate, id_list, money_or_hole, number,
    prob_value, record, reference, replaces_list, require, signed_money, signed_number, string,
    string_or_hole, Key, Value,
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
    (KwBasis, Some(bound)),
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
    (KwReplaces, Some(replaces_list)),
    (KwWhen, Some(bound)),
    (KwBasis, Some(bound)),
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
/// or waive it, the trigger parameter that designates its due date, and
/// the interpretation its least time rests on.
pub(crate) const DEADLINE_DECL_OPTS: &[Spec] = &[
    (KwExtendable, Some(boolean)),
    (KwExtensionAuthority, Some(string_or_hole)),
    (KwNote, Some(string)),
    (KwBearer, Some(reference)),
    (KwTrigger, Some(ref_or_call)),
    (KwExtend, Some(capability)),
    (KwWaive, Some(capability)),
    (KwDesignated, Some(reference)),
    (KwForum, Some(reference)),
    (KwInterpretation, Some(string)),
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
/// after_expiry, on Motion(), within Bound }` or `{ never cite "X" }`.
fn capability(p: &mut Parser<'_>) -> bool {
    field_list(p, CAPABILITY_FIELDS)
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

/// Value of a stake: signed money or `unvalued`.
pub(crate) fn stake_value(p: &mut Parser<'_>) -> bool {
    signed_money(p) || super::values::keyword_literal(p, &[KwUnvalued])
}

/// `patch Ident PatchFields` inside an instance: typed edge fields, each
/// replacing the base edge's, and the reserved [`guard_op`]s.
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
/// { VersionClause | Cite }` (SPEC §3.6): a typed operation on the patched
/// edge's guard with its authority, parsed in full so the rest of the file
/// parses, then rejected as L0010 until the operations are implemented.
/// `and` and `or` are reserved words the lexer already reports; the
/// `supersedes when` form is reported here.
fn guard_op(p: &mut Parser<'_>) -> bool {
    let combines = p.at_kw(KwWhen) && (p.nth_kw(1, KwAnd) || p.nth_kw(1, KwOr));
    if !combines && !p.at_kw(KwSupersedes) {
        return false;
    }
    let start = p.current_range().start();
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
    while super::clauses::version_clause(p) || super::values::cite(p) {}
    p.finish();
    if !combines {
        let end = p.last_end().max(usize::from(start));
        p.push_error(SyntaxError::new(
            DiagnosticCode::ReservedGuardOp,
            range(usize::from(start), end),
            format!(
                "{}: `supersedes when p` replaces a patched edge's guard, which a later \
                 change adds; patch the edge's other fields",
                crate::lexer::RESERVED_GUARD_OP
            ),
        ));
    }
    true
}
