# Remaining strict-profile L5002 holes in `lit/core`

Audit date: 2026-11-05. After correcting calendar cite proof windows, the release CLI's `litigate check --profile strict lit/core` reported 3 L5002 holes (exit 2). Calendar cites now begin proof at the earliest effective date of their entries. The three findings below remain because their source bindings do not pin section-specific amendment histories for the actual dates claimed.

## Search performed

- Inspected each bound source text and its `sources/PROVENANCE.md` entry. The text is available, but the tracked source corpus has no section-specific dated history proving it for the dates below.
- Searched tracked `sources/` files named `*version*` or `*history*`; none supplies the required histories for these three provisions. No law, amendment history, or date was inferred.

## Remaining diagnostics

| # | Diagnostic location | Cited provision | Proof begins | Bound source | Remaining gap |
|---:|---|---|---|---|---|
| 1 | `lit/core/states/hawaii.lit:11` | HRS § 8-1 | 2023-01-01 | `hawaii-holidays` → `sources/hi-holidays/hrs-8-1.txt` | The source has no pinned `## versions HRS § 8-1` amendment history proving the cited text from 2023-01-01 through its as-of date, 2026-11-02. |
| 2 | `lit/core/states/new-mexico.lit:16` | NMRA 1-006(A)(7) | 2024-12-31 | `nmra-1` → `sources/nm-state-civil/nmra-rule-set-1-civil-2026-sectioned.txt` | The source has no pinned `## versions NMRA 1-006` amendment history proving the cited text from 2024-12-31 through its as-of date, 2026-10-27. |
| 3 | `lit/core/states/utah.lit:17` | URCP 6(a)(6) | 2024-05-01 | `urcp` → `sources/ut-state-civil/utah-urcp-all-rules-sectioned.txt` | The source has no pinned `## versions URCP 6` amendment history proving the cited text from 2024-05-01 through its as-of date, 2026-10-26. |
