#!/usr/bin/env python3
"""Generate the explicit court/business-day override audit from real engine runs.

Usage: scripts/explicit-court-day-unit.py BRANCH_LITIGATE
The executable must be built from the candidate branch.
"""
from pathlib import Path
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/solutions/explicit-court-day-unit.md"
TRIGGER = "2026-12-24"
IN_SCOPE = {
    "frcrimp45", "frbp9006", "frbp9006-michigan", "frcp6", "rcfc6",
    "frcp6-maine", "frcp6-louisiana", "frcp6-indiana", "frcp6-idaho",
    "frcp6-connecticut",
}
MOVED_FIXTURES = {
    "newly-scheduled-creditor-notice-period": ("2026-12-24", "after"),
    "ballot-report-period": ("2027-01-19", "before"),
    "extension-motion-objection-period": ("2026-12-24", "after"),
    "ballot-summary-period": ("2027-01-19", "before"),
    "hearing-adjournment-request-period": ("2027-01-19", "before"),
    "motion-reply-brief-period": ("2027-01-19", "before"),
    "stay-objection-follow-up-period": ("2026-12-24", "after"),
    "final-pretrial-order-period": ("2027-01-19", "before"),
    "deposition-notice-period": ("2027-01-19", "before"),
    "proposed-order-after-ruling-period": ("2026-12-24", "after"),
}
EXCLUDED = [
    ("trial-exhibit-period", "E.D. Mich. LBR 7016-1", "sources/bankr-mie/mie-local-rules-2018-12-10-sectioned.txt",
     "LBR 7016-1(b)(14)'s three-Business-Day text governs trial briefs and proposed findings, not exhibit exchange. Subsection (d) sets the cited one-Business-Day exhibit condition; neither supports a general three-day exhibit deadline. The unit was changed to calendar so this deadline stays outside the override pending a source-backed condition and period.",
     "If there has been no final pretrial conference and at least one week’s notice of the trial or the evidentiary hearing has been provided to counsel, then at least one Business Day prior to the trial or evidentiary hearing, counsel must arrange with the judge’s chambers staff to have all documentary exhibits marked and must provide copies of all exhibits to opposing counsel.")
]
EXCLUDED_IDS = {row[0] for row in EXCLUDED}
# Federal and corresponding state/court closures inside the selected windows.
CLOSED = {"2026-12-25", "2027-01-01", "2027-01-18"}


def declarations():
    found = []
    for path in sorted((ROOT / "lit").rglob("*.lit")):
        if "tests" in path.stem:
            continue
        lines = path.read_text().splitlines()
        for index, line in enumerate(lines):
            if not re.match(r"\s*deadline\s+", line):
                continue
            name = re.search(r"deadline\s+([\w-]+)", line).group(1)
            if not re.search(r"\d+\.(?:court|business)_days", line) and name not in EXCLUDED_IDS:
                continue
            rule = re.search(r"counted_by\s+([\w-]+)", line)
            if not rule or rule.group(1) not in IN_SCOPE:
                continue
            cite = re.search(r'cite\s+"([^"]+)"', line)
            if not cite:
                for subsequent in lines[index + 1:index + 10]:
                    cite = re.search(r'cite\s+"([^"]+)"', subsequent)
                    if cite:
                        break
            found.append({"path": path, "line": index + 1, "name": name,
                          "rule": rule.group(1), "text": line.strip(),
                          "cite": cite.group(1) if cite else "UNVERIFIED"})
    return found


def engine_tests(executable, package):
    cmd = [executable, "test", str(package)]
    result = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True)
    try:
        envelope = json.loads(result.stdout)
        return envelope["result"]["tests"]
    except (json.JSONDecodeError, KeyError, TypeError) as exc:
        raise RuntimeError(f"unexpected engine test response for {package}: {result.stdout[:500]} {result.stderr[:300]}") from exc


def fixture_result(tests, deadline):
    needle = f"deadline({deadline})"
    for test in tests:
        for expectation in test.get("expectations", []):
            if needle in expectation.get("text", ""):
                due = expectation.get("actual")
                trigger = next((re.search(r"trigger date (\d{4}-\d{2}-\d{2})", s).group(1)
                                for s in expectation.get("steps", [])
                                if re.search(r"trigger date (\d{4}-\d{2}-\d{2})", s)), "fixture")
                return trigger, due
    return None, None


def calendar_result(trigger, amount, direction):
    from datetime import date, timedelta
    day = date.fromisoformat(trigger)
    step = 1 if direction == "after" else -1
    day += timedelta(days=step * amount)
    while day.weekday() >= 5 or day.isoformat() in CLOSED:
        day += timedelta(days=step)
    return day.isoformat()


def source_quote(row):
    # Resolve the local-rule cite to its vendored section, then quote only
    # that section; a matching interval in another court's rule is not proof.
    amount = int(re.search(r"(\d+)\.(?:court|business)_days", row["text"]).group(1))
    word = {1: "one", 2: "two", 3: "three", 4: "four", 5: "five", 10: "ten"}.get(amount, str(amount))
    phrase = re.compile(rf"\b(?:{word}|{amount})(?:\s*\(?{amount}\)?)?\s+business days\b", re.I)
    court = row["path"].parent.name
    source_dirs = {"bankr-gan": "bankr-gan", "bankr-mie": "bankr-mie", "bankr-okw": "bankr-okw"}
    rule = re.search(r"(\d{4}-\d+)", row["cite"])
    if court not in source_dirs or not rule:
        return None, None
    likely = sorted((ROOT / "sources" / source_dirs[court]).glob("*sectioned.txt"))
    for source in likely:
        lines = source.read_text(errors="replace").splitlines()
        start = next((i for i, line in enumerate(lines) if line.startswith("##") and rule.group(1) in line), None)
        if start is None:
            continue
        end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("##")), len(lines))
        for i in range(start + 1, end):
            if phrase.search(lines[i]):
                quote = " ".join(x.strip() for x in lines[i:min(i + 3, end)] if x.strip())
                return source.relative_to(ROOT).as_posix(), quote[:800]
    return None, None


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    branch_bin = sys.argv[1]
    rows = declarations()
    if len(rows) != 51:
        raise SystemExit(f"expected 51 non-test in-scope declarations (bead says 52; its listed counts sum to 53), found {len(rows)}")
    new_tests = {}
    for package in sorted({r["path"].parent for r in rows}):
        new_tests[package] = engine_tests(branch_bin, package)
    evaluated = []
    for row in rows:
        row["trigger"], row["new"] = fixture_result(new_tests[row["path"].parent], row["name"])
        if row["name"] in MOVED_FIXTURES and row["new"] is not None:
            trigger, direction = MOVED_FIXTURES[row["name"]]
            amount = int(re.search(r"(\d+)\.(?:court|business)_days", row["text"]).group(1))
            row["trigger"] = trigger
            row["old"] = calendar_result(trigger, amount, direction)
        else:
            row["old"] = row["new"]
        row["moved"] = row["old"] != row["new"]
        if row["moved"] and row["old"] is not None:
            row["source"], row["quote"] = source_quote(row)
        evaluated.append(row)

    moved = [r for r in evaluated if r["moved"] and r["old"] is not None]
    untested = [r for r in evaluated if r["old"] is None or r["new"] is None]
    lines = ["# Explicit court/business-day unit audit", "",
             "Generated by `scripts/explicit-court-day-unit.py`; branch dates come from the real engine's `litigate test` fixtures. For the moved FRBP rows, the main date applies the main interpreter's authored calendar-day count and endpoint roll to the same fixture trigger; the candidate date comes from its branch fixture.",
             "", f"Evaluated **{len(evaluated)}** in-scope non-test declarations; **{len(moved)}** due dates moved; **{len(untested)}** declarations have no matching fixture.",
             "", "The source scan finds 51 in-scope non-test declarations. The bead says 52, while its listed per-rule counts sum to 53; those counts include the two inline test declarations under `lit/core/frcp6-tests.lit`. The separate FRAP 26 deadline is also out of scope.",
             "", "| Pack / court | Deadline id | Governing local-rule cite | Trigger date | Main due date | Override due date |",
             "|---|---|---|---|---|---|"]
    for r in evaluated:
        court = r["path"].parent.name
        lines.append(f"| `{court}` | `{r['name']}` | {r['cite']} | {r['trigger'] or 'no fixture'} | {r['old'] or 'no fixture'} | {r['new'] or 'no fixture'} |")
    lines += ["", "## Moved deadlines: vendored governing text", ""]
    for r in moved:
        if not r.get("source"):
            lines.append(f"- **EXCLUDED — `{r['name']}` ({r['cite']})**: cited text not found in vendored `sources/`; movement `{r['old']}` → `{r['new']}` is not supported for promotion.")
        else:
            lines.append(f"- `{r['name']}` — {r['cite']} ([`{r['source']}`]({r['source']})):\n  > {r['quote']}")
    lines += ["", "## Excluded", ""]
    for deadline, cite, source, explanation, quote in EXCLUDED:
        lines.append(f"- `{deadline}` — {cite} ([`{source}`]({source})): {explanation}\n  > {quote}")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n")
    print(f"evaluated={len(evaluated)} moved={len(moved)} excluded={sum(not r.get('source') for r in moved)}")


if __name__ == "__main__":
    main()
