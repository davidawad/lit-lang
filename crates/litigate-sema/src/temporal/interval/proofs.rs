//! Kani proofs (`just verify-kani`) of the half-open window algebra
//! temporal checks rest on (SPEC §5.4). Window ends are drawn from a few
//! ordered dates so CBMC compares fixed-length text, not unbounded strings.
use super::Window;

const DATES: [&str; 4] = ["1999-12-31", "2000-01-01", "2009-12-01", "2024-02-29"];

fn date() -> &'static str {
    let i: usize = kani::any();
    kani::assume(i < DATES.len());
    DATES[i]
}

fn window() -> Window {
    let to = if kani::any() { Some(date()) } else { None };
    Window::new(date(), to)
}

/// A day lies in the intersection exactly when it lies in both windows,
/// and a shared day means the windows overlap.
#[kani::proof]
#[kani::unwind(12)]
fn intersection_is_conjunction() {
    let (a, b, day) = (window(), window(), date());
    let both = a.contains(day) && b.contains(day);
    let meet = a.intersect(&b);
    assert_eq!(meet.as_ref().is_some_and(|w| w.contains(day)), both);
    if both {
        assert!(a.overlaps(&b) && b.overlaps(&a));
    }
}

/// `within` is containment: every day of a window within another lies in
/// it; a window is empty exactly when it holds no day.
#[kani::proof]
#[kani::unwind(12)]
fn within_is_containment() {
    let (a, b, day) = (window(), window(), date());
    if a.within(&b) && a.contains(day) {
        assert!(b.contains(day));
    }
    // A non-empty window holds its own first day, one of the dates.
    assert_eq!(a.is_empty(), !DATES.iter().any(|d| a.contains(d)));
}
