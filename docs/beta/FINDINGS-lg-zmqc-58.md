# Unresolved strict-profile holes in `lit/core`

Audit date: 2026-11-05. This records the remaining L5002 holes from the strict
check described in `docs/beta/EPIC-AUDIT.md` (snapshot `f44da4dac`). No
amendment history is inferred from the current text or its retrieval date.

The 43 holes arise from state-law citations in `lit/core/states/*.lit`. The
state packages cite current state legal-holiday statutes, but their source
bindings pin only current text; the verifier requires a pinned, section-specific
`## versions <heading>` amendment history to prove that text was in force on
each declared effective date. The following sources were checked:

- `sources/*-holidays/PROVENANCE.md` and corresponding `*-legal-holidays.txt`
  extracts, including each extract's cited section and the source page's
  edition/amendment notes. These establish current text/source provenance but
  generally do not establish a complete sequence of effective versions.
- All tracked `sources/` paths matching `*versions*` or `*history*`. These
  cover federal rules, CFR, and specialized statutes; none is a complete
  section-by-section amendment history for the state statutes cited by these
  calendars.
- `lit/core/Lit.toml` state-holiday source bindings and `lit/core/states/*.lit`
  effective dates, to identify the exact state citations and proof windows.

Where the official state legislature's session laws or official code history
can be obtained, vendor the dated source and add a section-specific `## versions`
block. Where no authoritative history can be located, retain that individual
L5002 with the precise official and secondary public sources searched here and
the result; do not assert that no amendment occurred based only on the present
code text. This audit did not establish all 43 exact diagnostic-to-citation
mappings, so it does not claim those holes are cleared.
