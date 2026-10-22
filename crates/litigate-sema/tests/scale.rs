//! A stdlib-sized package (1,000 versioned deadlines, 200 declared
//! precedences, 200 defaults with prioritized exceptions) checks clean, and
//! the temporal and precedence pass over it stays near-linear: the budget
//! catches a regression to the quadratic scans it replaced (~0.75 s release).
mod legal;
use legal::{pkg, render, workspace, PRELUDE};
use litigate_sema::temporal::world::NoSource;
use litigate_sema::temporal::World;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

const BUDGET: Duration = if cfg!(debug_assertions) {
    Duration::from_millis(500)
} else {
    Duration::from_millis(50)
};

fn body() -> String {
    let mut s = String::new();
    for i in 0..1000 {
        let _ = writeln!(s, "deadline d{i}: 20.days from Served(complaint, Mail) counted_by c6 effective [2009-12-01, 2016-12-01)");
        let _ = writeln!(s, "deadline d{i}: 21.days from Served(complaint, Mail) counted_by c6 effective [2016-12-01, ..)");
    }
    for i in 0..200 {
        let _ = writeln!(s, "deadline o{i}: 7.days from Served(complaint, Mail) counted_by c6 effective [2018-01-01, ..) notwithstanding d{i}");
        let _ = writeln!(s, "default f{i} when has_event(Ordered()) => {i}");
        let _ = writeln!(
            s,
            "exception x{i} rebuts f{i} when has_event(Answered(complaint)) => 1 priority 1"
        );
        let _ = writeln!(
            s,
            "exception y{i} undercuts f{i} when has_event(Served(complaint, Mail)) priority 2"
        );
    }
    s
}

#[test]
fn a_stdlib_sized_package_checks_clean_within_budget() {
    let text = format!("{PRELUDE}{}", body());
    let (db, ws) = workspace(&[pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    )]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    legal::with_world(&db, ws, |world: &World<'_>| {
        assert!(world.decls.len() > 2_800);
        let best = (0..3)
            .map(|_| {
                let start = Instant::now();
                let found = litigate_sema::precedence::check(world, &NoSource, 0);
                assert!(found.is_empty(), "{found:?}");
                start.elapsed()
            })
            .min()
            .unwrap_or_default();
        eprintln!(
            "checked {} declarations in {best:?} (budget {BUDGET:?})",
            world.decls.len()
        );
        assert!(
            best < BUDGET,
            "temporal/precedence pass took {best:?}, budget {BUDGET:?}"
        );
    });
}
