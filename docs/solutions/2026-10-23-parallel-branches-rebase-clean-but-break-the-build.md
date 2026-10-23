---
module: workspace
tags: [integration, rebase, ci, litigate]
problem_type: workflow
---

# Parallel branches can rebase cleanly and still break the build

## Problem

During the litigate epic, about thirty agent branches were rebased one at a
time onto `litigate-epic`. Every rebase was textually clean, yet the epic
head stopped compiling twice:

- `litigate-cli` was written against an early `hir::Case` and
  `litigate_verify::Finding`; later branches added `Case.removals` and
  changed `Finding`'s fields. The CLI branch rebased with no conflict and
  then failed with E0063/E0560/E0004.
- `lit/core` passed its strict check against the CLI's temporary verifier,
  then failed once the real `litigate-verify` was wired in (its
  `section_heading` values carried a `## ` marker that lowering also adds).

## Root cause

A clean rebase proves only that no two branches edited the same lines. It
says nothing about type-level coupling across crates (a struct gaining a
field, an enum gaining a variant) or about two data conventions meeting for
the first time.

## Durable fact

Run the workspace gate (`cargo fmt --check`, `cargo clippy --workspace
--all-targets -D warnings`, `cargo nextest run --workspace`) on the
integrated head after every merge of a branch that was cut from an older
base, before the next branch builds on it. A green gate on the branch's own
base is not evidence for the integrated result.

Related convention settled here: `section_heading` in `Lit.toml` is written
without the markdown `## ` marker (SPEC §4.1); lowering adds it.
