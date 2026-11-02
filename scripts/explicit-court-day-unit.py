#!/usr/bin/env python3
"""Generate a paired main/branch deadline audit from real litigate runs.

Usage: scripts/explicit-court-day-unit.py MAIN_ROOT MAIN_LITIGATE BRANCH_LITIGATE
MAIN_ROOT must be a checkout of origin/main. The three audit fixture files are
briefly overlaid there so both binaries evaluate identical triggers; original
main files are restored even if a litigate invocation fails.
"""
from pathlib import Path
import json
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/solutions/explicit-court-day-unit.md"
IN_SCOPE = {
    "frcrimp45", "frbp9006", "frbp9006-michigan", "frcp6", "rcfc6",
    "frcp6-maine", "frcp6-louisiana", "frcp6-indiana", "frcp6-idaho",
    "frcp6-connecticut",
}
AUDIT_PACKAGES = ("bankr-gan", "bankr-mie", "bankr-okw")


def declarations():
    found = []
    for path in sorted((ROOT / "lit").rglob("*.lit")):
        if "tests" in path.stem:
            continue
        source_lines = path.read_text().splitlines()
        for index, line in enumerate(source_lines):
            if not re.match(r"\s*deadline\s+", line):
                continue
            name = re.search(r"deadline\s+([\w-]+)", line).group(1)
            if not re.search(r"\d+\.(?:court|business)_days", line):
                continue
            rule = re.search(r"counted_by\s+([\w-]+)", line)
            if not rule or rule.group(1) not in IN_SCOPE:
                continue
            period = re.search(r"\d+\.(?:court|business)_days\s+(before|after)\s+(.+?)\s+counted_by", line)
            if not period:
                raise SystemExit(f"cannot parse deadline event/direction: {path}:{index + 1}")
            cite = re.search(r'cite\s+"([^"]+)"', line)
            if not cite:
                for later in source_lines[index + 1:index + 10]:
                    cite = re.search(r'cite\s+"([^"]+)"', later)
                    if cite:
                        break
            found.append({"path": path, "name": name, "rule": rule.group(1),
                          "text": line.strip(), "cite": cite.group(1) if cite else "UNVERIFIED",
                          "direction": period.group(1), "event": period.group(2)})
    return found


def run_tests(executable, root, package):
    proc = subprocess.run([executable, "test", f"lit/{package}"], cwd=root,
                          text=True, capture_output=True)
    try:
        envelope = json.loads(proc.stdout)
        return envelope["result"]["tests"]
    except (json.JSONDecodeError, KeyError, TypeError) as exc:
        raise RuntimeError(f"{executable} test lit/{package} failed to return JSON: "
                           f"{proc.stdout[:500]} {proc.stderr[:500]}") from exc


def observations(tests, deadline):
    found = []
    for test in tests:
        for expectation in test.get("expectations", []):
            if f"deadline({deadline})" not in expectation.get("text", ""):
                continue
            trigger = None
            for step in expectation.get("steps", []):
                match = re.search(r"trigger date (\d{4}-\d{2}-\d{2})", step)
                if match:
                    trigger = match.group(1)
                    break
            if trigger:
                found.append((test.get("name", ""), trigger, expectation.get("actual")))
    return found


def source_quote(row):
    amount = int(re.search(r"(\d+)\.(?:court|business)_days", row["text"]).group(1))
    word = {1: "one", 2: "two", 3: "three", 4: "four", 5: "five", 10: "ten"}.get(amount, str(amount))
    phrase = re.compile(rf"\b(?:{word}|{amount})(?:\s*\(?{amount}\)?)?\s+business days\b", re.I)
    court = row["path"].parent.name
    rule = re.search(r"(\d{4}-\d+)", row["cite"])
    if court not in AUDIT_PACKAGES or not rule:
        return None, None
    for source in sorted((ROOT / "sources" / court).glob("*sectioned.txt")):
        lines = source.read_text(errors="replace").splitlines()
        start = next((i for i, line in enumerate(lines)
                      if line.startswith("##") and rule.group(1) in line), None)
        if start is None:
            continue
        end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("##")), len(lines))
        for i in range(start + 1, end):
            if phrase.search(lines[i]):
                quote = " ".join(x.strip() for x in lines[i:min(i + 3, end)] if x.strip())
                return source.relative_to(ROOT).as_posix(), quote[:800]
    return None, None


def paired_tests(main_root, main_bin, branch_bin, rows):
    originals = {}
    try:
        grouped = {}
        for row in rows:
            grouped.setdefault(row["path"], []).append(row)
        for branch_file, package_rows in grouped.items():
            rel = branch_file.relative_to(ROOT)
            main_file = main_root / rel
            originals[main_file] = main_file.read_bytes()
            shutil.copyfile(branch_file, main_file)
            originals[branch_file] = branch_file.read_bytes()
            additions = []
            for row in package_rows:
                trigger = "2027-01-19" if row["direction"] == "before" else "2026-12-24"
                additions.append(
                    f'\ntest "generated audit: {row["name"]}" cite "{row["cite"]}"\n'
                    f'{{\n  given {row["event"]} on {trigger}\n'
                    f'  expect deadline({row["name"]}) == 2000-01-01\n}}\n')
            for file in (main_file, branch_file):
                file.write_text(file.read_text() + "\n".join(additions))
        results = {}
        packages = sorted({r["path"].parent.name for r in rows})
        for package in packages:
            results[("main", package)] = run_tests(main_bin, main_root, package)
            results[("branch", package)] = run_tests(branch_bin, ROOT, package)
        return results
    finally:
        for path, data in originals.items():
            path.write_bytes(data)


def main():
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    main_root, main_bin, branch_bin = Path(sys.argv[1]).resolve(), sys.argv[2], sys.argv[3]
    rows = declarations()
    if len(rows) != 51:
        raise SystemExit(f"expected 51 non-test declarations, found {len(rows)}")
    tests = paired_tests(main_root, main_bin, branch_bin, rows)
    evaluated = []
    for row in rows:
        package = row["path"].parent.name
        old_tests = tests[("main", package)]
        new_tests = tests[("branch", package)]
        old_obs = observations(old_tests, row["name"])
        new_obs = observations(new_tests, row["name"])
        # Prefer the separately added holiday fixture; existing tests remain
        # intact and are still available as a fallback for declarations with
        # no holiday-specific fixture.
        prefer = lambda item: item[0].startswith("generated audit:")
        old_obs.sort(key=lambda item: (not prefer(item), item[1]))
        new_obs.sort(key=lambda item: (not prefer(item), item[1]))
        common = next(((o, n) for o in old_obs for n in new_obs if o[1] == n[1]), None)
        if common is None:
            raise SystemExit(f"no shared real-engine fixture for {package}/{row['name']}")
        old, new = common
        row.update(trigger=old[1], old=old[2], new=new[2], moved=old[2] != new[2])
        if row["moved"]:
            row["source"], row["quote"] = source_quote(row)
        evaluated.append(row)

    moved = [r for r in evaluated if r["moved"]]
    excluded = [r for r in moved if not r.get("source")]
    lines = ["# Explicit court/business-day unit audit", "",
             "Generated by `scripts/explicit-court-day-unit.py`. Every main and override due date below is the `actual` result from the real `litigate test` engine built from `origin/main` and this branch, run on the same trigger fixture. No due date is calculated by this script.", "",
             f"Evaluated **{len(evaluated)}** in-scope non-test declarations; **{len(moved)}** due dates moved; **{len(excluded)}** moved deadlines excluded for lack of vendored governing text.", "",
             "The source inventory contains 51 matching declarations (not 52). The count in the bead's rule-family list sums to 53 because it includes inline `lit/core/frcp6-tests.lit` examples; the separate FRAP 26 deadline is outside this inventory.", "",
             f"Audit totals: **{len(evaluated)} evaluated / {len(moved)} moved / {len(excluded)} excluded**.", "",
             "| Pack / court | Deadline id | Governing local-rule cite | Trigger date | Main due date | Override due date |",
             "|---|---|---|---|---|---|"]
    for r in evaluated:
        court = r["path"].parent.name
        lines.append(f"| `{court}` | `{r['name']}` | {r['cite']} | {r['trigger']} | {r['old']} | {r['new']} |")
    lines += ["", "## Moved deadlines: vendored governing text", ""]
    for r in moved:
        if not r.get("source"):
            lines.append(f"- **EXCLUDED — `{r['name']}` ({r['cite']})**: cited text was not found in vendored `sources/`; movement `{r['old']}` → `{r['new']}` is not promoted.")
        else:
            lines.append(f"- `{r['name']}` — {r['cite']} ([`{r['source']}`]({r['source']})):\n  > {r['quote']}")
    lines += ["", "## FINDING", "",
              "E.D. Mich. LBR 7016-1 source concern for CoS ([vendored rule text](sources/bankr-mie/mie-local-rules-2018-12-10-sectioned.txt)): subsection (b)(14)'s three-Business-Day requirement applies to trial briefs and proposed findings. Subsection (d) provides exhibit exchange at least one Business Day before trial only when no final pretrial conference occurred and counsel had a week's notice. The existing `trial-exhibit-period` declaration remains unchanged in this bead; its modeled three Business Days should be reviewed against that source. No deadline declaration was changed for this finding.",
              "",
              "> If there has been no final pretrial conference and at least one week’s notice of the trial or the evidentiary hearing has been provided to counsel, then at least one Business Day prior to the trial or evidentiary hearing, counsel must arrange with the judge’s chambers staff to have all documentary exhibits marked."]
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n")
    print(f"evaluated={len(evaluated)} moved={len(moved)} excluded={len(excluded)}")


if __name__ == "__main__":
    main()
