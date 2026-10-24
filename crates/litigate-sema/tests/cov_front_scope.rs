//! Scope lookups over hand-built signatures (SPEC §4.3): nominal types are
//! respelled across package boundaries, enums are found through imports.
use litigate_sema::db::parse::tree::ItemTree;
use litigate_sema::hir::Type;
use litigate_sema::resolve::scope::{Binding, Scope};
use litigate_sema::resolve::sig::{Signature, TypeSig};

fn sig(package: &str, pack_id: &str, enums: &[(&str, &[&str])]) -> Signature {
    Signature {
        module: format!("{package}.types"),
        package: package.to_owned(),
        pack_id: pack_id.to_owned(),
        types: enums
            .iter()
            .map(|(name, variants)| {
                let variants = variants.iter().map(|v| (*v).to_owned()).collect();
                ((*name).to_owned(), TypeSig::Enum(variants))
            })
            .collect(),
        ..Signature::default()
    }
}

fn named(n: &str) -> Type {
    Type::Named(n.to_owned())
}

#[test]
fn types_from_another_package_are_requalified() {
    let own = sig("app", "app-pack", &[]);
    let dep = sig("dep", "dep-pack", &[]);
    let scope = Scope::new(&own, &ItemTree::default(), &[], Vec::new());
    let written = Type::Record(vec![
        ("bare".to_owned(), named("Outcome")),
        ("ours".to_owned(), named("app-pack::Mine")),
        ("theirs".to_owned(), named("third::Other")),
        ("dist".to_owned(), Type::Distribution("Outcome".to_owned())),
        ("list".to_owned(), Type::List(Box::new(named("Outcome")))),
        ("plain".to_owned(), Type::Date),
    ]);
    let seen = Type::Record(vec![
        ("bare".to_owned(), named("dep-pack::Outcome")),
        ("ours".to_owned(), named("Mine")),
        ("theirs".to_owned(), named("third::Other")),
        (
            "dist".to_owned(),
            Type::Distribution("dep-pack::Outcome".to_owned()),
        ),
        (
            "list".to_owned(),
            Type::List(Box::new(named("dep-pack::Outcome"))),
        ),
        ("plain".to_owned(), Type::Date),
    ]);
    assert_eq!(scope.localize(&dep, &written), seen);
    assert_eq!(scope.localize(&own, &written), written);
}

#[test]
fn enums_are_found_through_imports() {
    let own = sig("app", "app-pack", &[("Local", &["x"])]);
    let imported = sig("app", "app-pack", &[("Outcome", &["granted", "denied"])]);
    let mut scope = Scope::new(&own, &ItemTree::default(), &[], Vec::new());
    scope.bindings.push(Binding {
        path: imported.module.clone(),
        names: None,
        sig: &imported,
    });
    assert_eq!(
        scope.enum_variants("Outcome"),
        Some(&["granted".to_owned(), "denied".to_owned()][..])
    );
    assert_eq!(scope.enums_covering(&["granted"]), ["Outcome"]);
    assert_eq!(scope.enums_covering(&[]), Vec::<String>::new());
}

#[test]
fn an_import_list_limits_the_enums_a_record_can_denote() {
    // T8: `import app.types.{Other}` does not make `Outcome` visible.
    let own = sig("app", "app-pack", &[]);
    let imported = sig(
        "app",
        "app-pack",
        &[("Outcome", &["granted", "denied"]), ("Other", &["granted"])],
    );
    let mut scope = Scope::new(&own, &ItemTree::default(), &[], Vec::new());
    scope.bindings.push(Binding {
        path: imported.module.clone(),
        names: Some(vec!["Other".to_owned()]),
        sig: &imported,
    });
    assert_eq!(scope.enums_covering(&["granted"]), ["Other"]);
    assert_eq!(scope.enums_covering(&["denied"]), Vec::<String>::new());
}
