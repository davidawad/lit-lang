# The deadline domain: change files and agreement report

Status: lg-fq2.6, measured 2026-10-27. Domain: federal deadline
computation, FRCP 6 and FRAP 26 and the periods they count (lg-fq2;
CHANGE.md). Every rate below is a test: `crates/litigate-cli/tests/`
`change_validation.rs` (VALIDATION.md) and `change_files.rs` (this file).

## The change files

| Instrument | Where | Class | Changes |
|---|---|---|---|
| FRCP 6, 1963-2016 (orders of 1963, 1985, 2001, 2005, 2009, 2016) | `lit/core/frcp6.lit` | base, one version per order | short-period rule, service days, inaccessible clerk, hours, months and years |
| FRAP 26, 1968-2016 (orders of 1967, 1989, 1996, 2002, 2009, 2016) | `lit/core/frap26.lit` | base, one version per order | the same, for the courts of appeals |
| FRCP 23(f), 50(b), 59(b), 59(e) before 2009 | `docs/litigate/change/frcp-base` | base | 10 days; 6(b)(2) forbids extending 50(b), 59 |
| Order of March 26, 2009 (FRCP) | `docs/litigate/change/frcp-2009` | amendment | 23(f) 14 days; 50(b), 59(b), 59(e) 28 |
| FRAP 4(a)(1), 4(a)(5), 4(a)(6) as of 1991 | `docs/litigate/change/frap-base` | base | 30, 30 after, 7 to move to reopen |
| Order of April 25, 2005 (FRAP) | `docs/litigate/change/frap-2005` | amendment | 4(a)(6): formal Rule 77(d) notice opens the 7 days |
| Order of March 26, 2009 (FRAP) | `docs/litigate/change/frap-2009` | amendment | 4(a)(6): 14 days |
| S.D.N.Y./E.D.N.Y. Local Civil Rule 6.1 | `docs/litigate/examples/change/local-rule` | local-rule | 7-day discovery opposition |
| Judge Breyer's General Standing Order | `docs/litigate/examples/change/standing-order` | standing-order | courtesy copy the next court day |
| COFC General Order 2025-02 | `docs/litigate/examples/change/general-order` | general-order | November 28, 2025 closed (recorded, not counted) |
| Cancino-Perez, D. Colo., ECF 42 | `docs/litigate/examples/change/case-order` | case-order | a designated amendment date |

Each layer cites its vendored text (`sources/change-*.txt`,
`sources/PROVENANCE.md`); `docs/litigate/change/worlds/` holds the actual
world at each circuit the rulings come from, and `docs/litigate/change/`
each ruling as a matter on the opinion's own dates.

## Agreement

| What | Agree | Disagree | Not counted |
|---|---|---|---|
| 2009 Committee Note examples (VALIDATION.md §2) | 23 of 25 | 0 | 2 (A13, A14: D5) |
| Timeliness rulings, golden tests (VALIDATION.md §3) | 35 of 35 | 0 | 0 |
| Rulings replayed through the change files (`litigate worlds`) | 18 of 19 | 1 (Circuitronix, D2) | 0 |
| Circuit split `frap-4a6-notice`, outcome per circuit | 8 golden tests | 0 | 0 |
| Circuit split `aedpa-grace-end` | the April 24 side | 0 | April 23 side (D3) |
| Replay before an overruling (Bowles of Thompson) | 1 of 1 | 0 | 0 |

The six AEDPA rulings, refused before this bead, count now: FRCP 6 as
amended in 1963 and 1985 counts a year to its anniversary (Ross v. Artuz,
150 F.3d 97, 103 (2d Cir. 1998); VALIDATION.md D1). Nine rulings compare
against a corpus DERIVED date; Burns agrees on its holding only. The
replay counts the 19 rulings whose period a layer declares; each gives the
golden tests' due date, every one but Circuitronix the court's outcome.

## Open failures

1. **D2, inaccessible days (language).** No event form states a closed
   clerk's office, so Circuitronix's matter is late where the court, the
   courthouse closed July 5, 2019, held it timely; the golden test counts
   it on the engine. Same gap: COFC General Order 2025-02's closure is
   recorded but no counting rule reads a layer's calendar.
2. **Counting rules are not layers (language).** A `counting` declaration
   versions only within its package (another package's `supersedes` of
   `frcp6` is L1001/L1002), so the orders amending FRCP 6 and FRAP 26 are
   versions in `lit/core`, each with its order's date and Note, not
   separate amendment packages.
3. **D3, precedent and triggers (language).** No form lets a binding
   precedent choose a deadline's trigger or counting; the split's reading
   per circuit is the test's. The April 23 AEDPA reading counts the
   trigger day, which no counting rule states.
4. **"Whichever is earlier" (language).** An act is judged against one
   period: FRAP 4(a)(6)(B)'s 180-day bound is declared beside the 7-day
   window, and Vencor's untimely motion is counted by its golden test.
5. **Pending cases (model).** The amendments' "insofar as just and
   practicable" clause is modelled for FRCP 6 (FRCP 86(a)(2)(B)
   transitions), not for the periods of `frcp-2009`.
6. **Not layered.** E.D. Cal. L.R. 293(a) (Kyle) has no vendored text or
   adoption date, so it is a golden test, not a local-rule layer; FRAP 26
   has no month or year provision (no collected ruling counts one).
7. **D4-D9** (VALIDATION.md §5): a date certain rolled under FRCP 6 (D4,
   pinned), times of day (D5), and the source ambiguities D6-D9.
