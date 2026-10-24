//! The check pipeline's salsa queries (design review §4.4) against the
//! direct computation over every loaded package, on the whole `lit/`
//! corpus: a package verified through `package_verdict`, which reads only
//! the packages it can see, has exactly the verdict it has when every
//! loaded package is in view.
use litigate_base::Profile;
use litigate_cli::clock::Clocks;
use litigate_cli::queries::{package_verdict, Build};
use litigate_cli::verdict;
use litigate_cli::ws::{discover, Session};
use litigate_lower::Options;
use std::path::Path;

#[test]
fn every_corpus_verdict_equals_the_verdict_over_every_package() {
    let lit = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lit");
    let targets = discover(&[lit.to_string_lossy().into_owned()]).expect("lit/ is found");
    let session = Session::load(&targets, &[]).expect("lit/ loads");
    let all = session.hir();
    let clocks = Clocks::new(&all);
    let options = Options::default();
    let build = Build::of(&session.db, Profile::Dev, &options);
    let mut links = 0;
    for pkg in &session.packages {
        let direct = verdict::package(
            &session.db,
            session.ws,
            pkg,
            &all,
            &clocks,
            Profile::Dev,
            &options,
        );
        let queried = package_verdict(&session.db, session.ws, pkg.input, build);
        assert_eq!(queried, &direct, "{}", pkg.name);
        links += usize::from(pkg.check.package.as_ref().is_some_and(|p| p.manifest.links));
    }
    assert!(session.packages.len() > 10 && links == 1);
}
