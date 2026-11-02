# Lowering expectations for lg-zmqc.28

## `next_day` observance

Keep `observed next_day` lowering to the engine's `NextDay` observance. The
cause was commit `0536dbbd0` (lg-zlqp), which added the engine semantic and
documented it in SPEC §5.3 and PACK_SCHEMA. The public authority for the
Maryland calendar is Md. Code, Gen. Provis. § 1-111(b), vendored in
`sources/md-holidays/md-legal-holidays.txt` (lines 48–53): a legal holiday
falling on Sunday is observed on the next Monday. This produces Sunday →
Monday, while Saturday and weekday dates remain unchanged. CoS approved this
scope on 2026-11-02; the Maryland calendar cites § 1-111(b). Other state
calendars must be supported by their own cited authority.

## Business-day lowering and basis

Keep the LW3 refusal for business-day periods and edge deadlines: they have no
engine lowering and must not be silently counted as court days. Keep an
explicit source `Hole` as `Hole` when a scenario prior replaces its
probability; the scenario value does not repair a missing source probability.
These are the restored expectations from the CoS ruling and have dedicated
fixtures. They are outside the approved `next_day` change.
