//! Nominal types keep one identity across packages: an enum or event
//! declared in one package is the same type when another package imports
//! it directly or through an alias or event of a third package.
mod common;

use common::{package, render, workspace};

const FRCP: &str = "module frcp
start s
enum Document { complaint, answer }
enum Method { Personal, Mail }
event Served(document: Document, method: Method)
type Outcome = Dist<Method>
default mail-share when true => { Personal: 1/4, Mail: 3/4 }
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
";

/// Re-exports frcp's `Document` through an alias and an event of its own.
const MID: &str = "module mid
import frcp.{ Document, Method }
start m
type Paper = Document
event Filed(paper: Paper, method: Method)
terminal m: \"M\"
";

const APP: &str = "module app
import frcp.{ Served, Document, Method, c6 }
import mid.{ Filed, Paper }
start a
deadline answer-due: 21.days after Served(complaint) counted_by c6
deadline reply-due: 14.days after Filed(answer, Mail) counted_by c6
default same when true => Document.complaint == Paper.complaint
state a: \"A\" {
  served \"Served\" -> b { event Served(complaint, Personal) }
  filed \"Filed\" -> b { event Filed(answer) }
}
terminal b: \"B\"
test \"due\" {
  given Served(complaint, Mail) on 2025-03-03
  expect deadline(answer-due) == 2025-03-27
}
";

#[test]
fn imported_nominal_types_have_one_identity() {
    let (db, ws) = workspace(&[
        package("app", &["frcp", "mid"], &[("app.lit", APP)]),
        package("mid", &["frcp"], &[("mid.lit", MID)]),
        package("frcp", &[], &[("frcp.lit", FRCP)]),
    ]);
    let out = render(&db, ws);
    assert!(out.is_empty(), "{out:#?}");
    let checks = litigate_sema::check_workspace(&db, ws);
    assert!(checks.iter().all(|c| c.package.is_some()));
}

#[test]
fn distinct_packages_keep_distinct_types() {
    let other = "module other\nstart o\nenum Document { complaint, answer }\nterminal o: \"O\"\n";
    let app = "module app\nimport frcp.{ Served }\nimport other.{ Document }\nstart a\nstate a: \"A\" { _ \"go\" -> b { event Served(Document.complaint) } }\nterminal b: \"B\"\n";
    let (db, ws) = workspace(&[
        package("app", &["frcp", "other"], &[("app.lit", app)]),
        package("other", &[], &[("other.lit", other)]),
        package("frcp", &[], &[("frcp.lit", FRCP)]),
    ]);
    let out = render(&db, ws);
    assert!(
        out.iter().any(|d| d.contains("L2001")
            && d.contains("frcp::Document")
            && d.contains("other::Document")),
        "{out:#?}"
    );
}

#[test]
fn reexported_types_resolve_without_importing_their_package() {
    let app = "module app
import mid.{ Filed, Paper }
start a
default same when true => Paper.complaint == Paper.answer
state a: \"A\" { _ \"go\" -> b { event Filed(complaint, Mail) } }
terminal b: \"B\"
";
    let (db, ws) = workspace(&[
        package("app", &["mid"], &[("app.lit", app)]),
        package("mid", &["frcp"], &[("mid.lit", MID)]),
        package("frcp", &[], &[("frcp.lit", FRCP)]),
    ]);
    let out = render(&db, ws);
    assert!(out.is_empty(), "{out:#?}");
}
