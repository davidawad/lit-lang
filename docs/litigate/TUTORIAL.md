# litigate in four steps

This walkthrough writes a rule, tests it, writes a case, and runs solve on
it. It takes about ten minutes. `SPEC.md` is the reference for every
construct used here, and `litigate describe` prints the whole language as
JSON. Run everything from the repository root. `litigate` is
`cargo run -q -p litigate-cli --` (or `cargo install --path
crates/litigate-cli`).

The example is a toy forum with one rule: a defendant must answer within
21 days of service (FRCP 12(a)(1)(A)(i)), counted under FRCP 6. Every
command and output below was run against this repository.

## 1. Write a rule

A package is a directory with a `Lit.toml`. Create `lit/my-court/Lit.toml`:

```toml
[package]
name = "my-court"
version = "0.1.0"
language = 1
pack_id = "my-court"          # the litgraph pack id; node ids become my-court::<node>
forum = "frcp"                # a district court: FRCP 6 counts its periods (SPEC §5.3)
authority = "FederalRule"     # default rank of every declaration (SPEC §5.5)
title = "My court: answer or default"
in_force = "2024-12-01"       # default effective start: the date of the text cited
known_at = "2024-12-01"       # default corpus date

[package.roles]               # actor -> self | opponent | nature
Plaintiff = "self"
Defendant = "opponent"
Court = "nature"

[dependencies]
core = { version = "^1.0.0", path = "../core" }

[sources.frcp]                # every cite must resolve to a span of this text
title = "Federal Rules of Civil Procedure"
url = "https://www.law.cornell.edu/rules/frcp"
path = "sources/frcp.txt"     # the vendored text cites resolve against,
path_sha256 = "527fb9fbfe1665bdbdc5565ea5614b299e16b07b07edba803aae6eb5f957b990"
as_of = "2024-12-01"          # pinned by its SHA-256; the expression's date
cite_prefixes = ["FRCP"]
section_heading = "Rule {n}"
```

Then write `lit/my-court/answer.lit`: the duty and its deadline, the
procedure graph it drives, and the terminals with their payoffs:

```lit
module my-court.answer

import core.frcp6.{ frcp6 }
import core.time.{ ServiceMethod }

start served

/// Service of the summons and complaint on the defendant.
event Served(method: ServiceMethod)

/// A defendant must serve an answer within 21 days after being served
/// with the summons and complaint.
rule duty answer-duty cite "FRCP 12(a)(1)(A)(i)"
{
  deadline answer-period: 21.days from Served() counted_by frcp6
}

decision served: "Answer or let the deadline pass" by Defendant cite "FRCP 12(a)(1)(A)(i)"
{
  answer "File an answer" -> answered { effort 8.hours, within answer-period }
  default "Let the deadline pass" -> defaulted { valence bad }
}

chance answered: "Merits" by Court cite "FRCP 56(a)"
{
  win "Judgment for the plaintiff" -> plaintiff-wins p 2/5
  lose "Judgment for the defendant" -> defendant-wins p 3/5
}

terminal defaulted: "Default judgment" outcome [judgment, win] cite "FRCP 55(b)"
{
  payoff 100_000.usd
}

terminal plaintiff-wins: "Judgment for the plaintiff" outcome [judgment, win] { payoff 100_000.usd }
terminal defendant-wins: "Judgment for the defendant" outcome [judgment, loss] { payoff 0.usd }
```

`frcp6` and `ServiceMethod` come from `lit/core`, the standard library of counting rules,
holiday calendars and shared vocabulary. A deadline is a period of its package's `forum`, and
its counting rule must be one that forum's rules use: `frcp6` declares `forums [frcp]`, so this
package's forum is `frcp`, and counting a period of another forum by it is L3009 (a period set
by another forum's rules, such as a FRAP 4 appeal period, names that forum with `{ forum frap }`). Pin the dependency, put the file in canonical form,
and check it:

```bash
litigate lock lit/my-court          # writes lit/my-court/Lit.lock
litigate fmt lit/my-court           # canonical layout (strict rejects anything else, L9004)
litigate check lit/my-court         # types, references, cites, verification obligations
```

`check` reports every problem as a diagnostic with a stable code, a span,
and usually a machine-applicable fix (`litigate fix lit/my-court`).
`litigate explain <code>` gives the long explanation of any code. A
probability you do not know yet is a hole, `p ?("no data yet")`. Holes
compile under the default `dev` profile with a warning; the repository gate
(`just lit-check`) runs that profile. Under `--profile strict`, the release
bar, they are errors (L5002), as are cites that resolve to no source. A cite whose vendored text is dated
after its declaration takes effect is a hole too, unless the section's
vendored amendment history shows the text unamended since (SPEC §7): that
is why `in_force` above is the date of the vendored FRCP text.

## 2. Test it

Inline tests pin what a rule computes. Add them to `answer.lit`:

```lit
test "answer due 21 days after personal service" cite "FRCP 12(a)(1)(A)(i)"
{
  given Served(Personal) on 2025-03-03
  expect deadline(answer-period) == 2025-03-24
}

test "a period ending on a Saturday rolls to Monday" cite "FRCP 6(a)(1)(C)"
{
  given Served(Personal) on 2025-03-08
  expect deadline(answer-period) == 2025-03-31
}

test "mail service adds three days" cite "FRCP 6(d)"
{
  given Served(Mail) on 2025-03-03
  expect deadline(answer-period) == 2025-03-27
}
```

```bash
litigate test lit/my-court          # {"passed":3,"failed":0,...}, each with its counting steps
```

Each result lists the steps the clock took, e.g. `FRCP 6(a): exclude the
trigger date 2025-03-03`. A wrong expectation fails with the date the rule
actually produces:

```
[L5006] Error: test "a period ending on a Saturday rolls to Monday": `deadline(answer-period) == 2025-03-29` does not hold: found "2025-03-31"
```

`check` runs the tests too (obligation `tests-pass`). `rule-without-test`
(L9001) flags every deadline and counting rule that no test exercises.

## 3. Write a case

A `.case` file is one matter: who the parties are, what has happened, and
what is at stake. Its posture is never asserted. It is derived by replaying
the event history against the rules. Create `cases/acme-v-roadrunner.case`:

```lit
case acme-v-roadrunner
uses my-court ^0.1.0
as_of 2025-04-01
summary "Acme served Roadrunner; will Roadrunner answer or default?"
notes "Tutorial matter. Stakes are illustrative."
perspective acme

parties {
  acme: Plaintiff Corporation
  roadrunner: Defendant Individual
}

events {
  service: 2025-03-03 Served(Personal) on roadrunner
}

stakes { my-court::defaulted: 100_000.usd }
analysis { params: { rate: 650.usd / 1.hours, opp_rate: 550.usd / 1.hours } }
```

Each event line starts with an id (`service:`) that you choose. It names
the event in the timeline, and it stays the same if you later correct the
event's date. An event without an id gets an info note (L9006). Its fix,
`litigate fix cases --code L9006`, writes a generated id such as
`served-1:`.

```bash
litigate lock cases                 # pins my-court in cases/Lit.lock
litigate check lit/my-court cases   # replays the history: an undeclared event (L1002), events
                                    # out of order or past their deadline (L4001-L4008) fail
```

## 4. Run solve

```bash
litigate run cases/acme-v-roadrunner.case --op solve
```

`run` compiles the matter to a litgraph scenario and runs the engine op.
The result is the engine's usual envelope. Here it is `value: 34800`, with
a `best_line` in which the defendant answers (`deadline_days: 21`) and the
merits go against the plaintiff with p 0.6. Any engine op works the same
way: `--op chain`, `--op '{"op":"simulate","runs":2000}'`, and so on.

`litigate build lit/my-court cases/acme-v-roadrunner.case --out build`
writes what was compiled: `packs/my-court.json` (the pack the engine
loads), `scenarios/acme-v-roadrunner.json`, a `*.map.json` source map from
every JSON pointer back to its declaration and cite, and the content-hashed
litigatable. `litigate where my-court::answer` finds a declaration from any
id.

## Making it part of the repository

The repository's packs, `links.json` and named scenarios are built from
`lit/` and `cases/` at `cargo build` time by `crates/litgraph-packs`
(SPEC §9.4); they are build outputs, never committed. To ship a new forum,
add its package (and any named matter) to `crates/litgraph-packs/build.args`,
then:

```bash
just lit-sync                       # relock; build packs/ and scenarios/ into target/lit-build/
just lit-sync-check                 # what CI runs: fails on a stale Lit.lock
```

Commit the source and its `Lit.lock` changes; the JSON is not committed. To
change an existing forum, edit its `lit/<forum>/` package. Run `just
lit-sync` before the edit; afterwards `just lit-diff` shows, by JSON pointer,
what the edit changes in the built files (SPEC §8.6).
