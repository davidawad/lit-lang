//! The obligation options of a deadline (`bearer`, `trigger`, `extend`,
//! `waive`) and the effect clause of an event (`removes`, `extends`,
//! `waives`), lowered by `types::build` (SPEC §5.2, §5.3): what lowers,
//! and one diagnostic path per line of source.
mod common;
use common::{check_body, package, workspace};
use litigate_sema::hir::Declaration;

/// The declarations of an error-free single-module package with `body`.
fn declarations(body: &str) -> Vec<Declaration> {
    let text = format!("{}{body}\n", common::PRELUDE);
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let errors: Vec<String> = common::render(&db, ws)
        .into_iter()
        .filter(|d| !d.contains(" L9"))
        .collect();
    assert_eq!(errors, Vec::<String>::new());
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter().cloned())
        .collect()
}

#[test]
fn obligation_options_and_effects_lower() {
    let found: Vec<_> = declarations(
        "rule duty answer-duty {\n\
  deadline a: 21.days from Served(complaint) counted_by c6 {\n\
    bearer Defendant\n\
    trigger latest\n\
    extend { by Court, max 30.days, after_expiry }\n\
    waive { by Plaintiff }\n\
  }\n\
}\n\
deadline b: 14.days from Served(answer) counted_by c6 { trigger later_of(a) }\n\
deadline e: 14.days from Served(answer) counted_by c6 { trigger each }\n\
deadline f: 14.days from Served(answer) counted_by c6 { trigger first, extendable true, extend { by Court, max 1.days } }\n\
deadline g: 14.days from Served(answer) counted_by c6 { extendable true, extend { by Court, discretion GoodCause }, extend { by Court, discretion ExcusableNeglect, after_expiry } }\n\
deadline h: 14.days from Served(answer) counted_by c6 { extendable false, extend { never } }\n\
event Moved(until: Date, option: String) extends a to until\n\
event Released() waives b\n\
event Dropped(option: String) removes option\n\
event Plain()",
    )
    .into_iter()
    .filter(|d| matches!(d, Declaration::Deadline(_) | Declaration::Event(_)))
    .collect();
    insta::assert_debug_snapshot!(found);
}

#[test]
fn obligation_options_report_each_bad_value() {
    let out = check_body(
        "deadline a: 21.days from Served(complaint) counted_by c6 { bearer Nobody }\n\
deadline b: 21.days from Served(complaint) counted_by c6 { trigger sometimes }\n\
deadline c: 21.days from Served(complaint) counted_by c6 { trigger later_of(c) }\n\
deadline d: 21.days from Served(complaint) counted_by c6 { trigger later_of(a, b) }\n\
deadline e: 21.days from Served(complaint) counted_by c6 { trigger later_of(1) }\n\
deadline f: 21.days from Served(complaint) counted_by c6 { trigger earlier_of(a) }\n\
deadline g: 21.days from Served(complaint) counted_by c6 { trigger later_of(c6) }\n\
deadline h: 21.days from Served(complaint) counted_by c6 { trigger 3 }\n\
deadline i: 21.days from Served(complaint) counted_by c6 { trigger first.x }\n\
deadline j: 21.days from Served(complaint) counted_by c6 { extend { by Court } }\n\
deadline k: 21.days from Served(complaint) counted_by c6 { extend { by Court, max 3.court_days } }\n\
deadline l: 21.days from Served(complaint) counted_by c6 { extend { by Court, max 3.hours } }\n\
deadline m: 21.days from Served(complaint) counted_by c6 { extend { max 3.days } }\n\
deadline n: 21.days from Served(complaint) counted_by c6 { waive { by Court, max 3.days } }\n\
deadline o: 21.days from Served(complaint) counted_by c6 { waive { after_expiry } }\n\
deadline p: 21.days from Served(complaint) counted_by c6 { extendable false, extend { by Court, max 3.days } }\n\
deadline q: 21.days from Served(complaint) counted_by c6 { extendable true, extend { by Court, max 3.days }, waive { by Courtt } }\n\
deadline r: 21.days from Served(complaint) counted_by c6 { trigger x.later_of(a) }\n\
deadline u: 21.days from Served(complaint) counted_by c6 { trigger later_of(a.b) }\n\
deadline v: 21.days from Served(complaint) counted_by c6 { extend { never, by Court } }\n\
deadline w: 21.days from Served(complaint) counted_by c6 { extend { never }, extend { by Court, discretion GoodCause } }\n\
deadline x: 21.days from Served(complaint) counted_by c6 { extend { by Court, max 3.days, discretion GoodCause } }\n\
deadline y: 21.days from Served(complaint) counted_by c6 { extendable true, extend { never } }\n\
deadline z: 21.days from Served(complaint) counted_by c6 { waive { by Court, discretion GoodCause, never } }\n\
deadline aa: 21.days from Served(complaint) counted_by c6 { extend { by Court, discretion 3 } }\n\
deadline ab: 21.days from Served(complaint) counted_by c6 { extend { discretion GoodCause } }\n\
deadline ac: 21.days from Served(complaint) counted_by c6 { extend { by Court, max } }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn event_effects_report_each_bad_clause() {
    let out = check_body(
        "deadline a: 21.days from Served(complaint) counted_by c6\n\
event E1(option: String) removes nothing\n\
event E2(option: Date) removes option\n\
event E3(until: Date) extends a\n\
event E4(until: Date) to until\n\
event E5(until: Date) extends nowhere to until\n\
event E6(until: String) extends a to until\n\
event E7(until: Date) extends c6 to until\n\
event E8(option: String) removes option waives a\n\
event E9() waives zzz\n\
event E10(until: Date) removes until.x\n\
event E11(until: Date) extends a to until.x\n\
event E12(option: String) waives a to option\n\
event E13(option: String) removes option removes option\n\
event E14() waives",
    );
    insta::assert_debug_snapshot!(out);
}
