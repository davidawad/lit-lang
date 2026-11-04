#!/usr/bin/env python3
"""Compare Bankr. N.D. Ga. deadline dates on a base ref and this checkout."""
from datetime import date, timedelta
import argparse
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
HOLIDAYS = {
    date(2026, 4, 3): "Good Friday (O.C.G.A. § 1-4-1(a)(2))",
    date(2026, 11, 27): "State Holiday (O.C.G.A. § 1-4-1(a)(2))",
    date(2026, 12, 24): "Washington's Birthday observed (O.C.G.A. § 1-4-1(a)(2))",
}
DECL = re.compile(
    r'^deadline\s+([\w-]+):\s*(\d+)\.([\w]+)\s+(after|before)\s+'
    r'(\w+)\(\).*?cite\s+"([^"]+)"', re.M)


def calculate(binary, tree, specs, triggers):
    # Inline deadline tests run through the compiler's real clock evaluator.
    # Each expectation exposes its actual computed date in JSON, allowing one
    # invocation to cover the full Cartesian product, including before rules.
    path = tree / "lit/bankr-gan/local-rules.lit"
    original = path.read_text()
    generated = []
    keys = []
    for trigger in triggers:
        for deadline, (_, _, _, event, cite) in specs.items():
            name = f"matrix-{deadline}-{trigger.isoformat()}"
            generated.append(
                f'test "{name}" cite "{cite}" {{\n'
                f'  given {event}() on {trigger.isoformat()}\n'
                f'  expect deadline({deadline}) != 0001-01-01\n'
                f'  expect deadline({deadline}).nominal != 0001-01-01\n'
                f'}}\n')
            keys.append((name, deadline, trigger))
    path.write_text(original + "\n" + "\n".join(generated))
    try:
        p = subprocess.run(
            [binary, "test", str(tree / "lit/bankr-gan"), "--lib", str(tree / "lit"),
             "--format", "json"],
            cwd=tree, capture_output=True, text=True)
        if p.returncode:
            raise RuntimeError(f"litigate test failed in {tree}: {p.stderr}{p.stdout[-2000:]}")
        response = json.loads(p.stdout)
        if not response.get("ok"):
            raise RuntimeError(f"litigate test returned an error in {tree}: {response}")
        tests = {test["name"]: test for test in response["result"]["tests"]}
        values = {}
        for name, deadline, trigger in keys:
            test = tests.get(name)
            if test is None or not test["passed"] or len(test["expectations"]) != 2:
                raise RuntimeError(f"generated clock test failed or missing: {name}")
            actual = [expectation.get("actual") for expectation in test["expectations"]]
            due, nominal = (date.fromisoformat(str(value)) for value in actual)
            values[(deadline, trigger)] = (due, nominal)
        return values
    finally:
        path.write_text(original)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--old-ref", default="origin/main")
    parser.add_argument("--litigate", default=os.environ.get("LITIGATE", "litigate"))
    args = parser.parse_args()
    old_tree = None
    try:
        specs = {m.group(1): (int(m.group(2)), m.group(3), m.group(4), m.group(5), m.group(6))
                 for m in DECL.finditer((ROOT / "lit/bankr-gan/local-rules.lit").read_text())}
        if not specs:
            raise RuntimeError("no local deadline declarations found")
        old_tree = Path(tempfile.mkdtemp(prefix="bankr-gan-old-"))
        archive = subprocess.run(["git", "archive", args.old_ref], cwd=ROOT,
                                 capture_output=True, check=True)
        with tarfile.open(fileobj=io.BytesIO(archive.stdout)) as tar:
            tar.extractall(old_tree, filter="data")
        triggers = []
        day = date(2026, 1, 1)
        while day <= date(2027, 12, 31):
            triggers.append(day)
            day += timedelta(days=1)

        from concurrent.futures import ThreadPoolExecutor
        # The two independent compile/evaluation runs use separate processes.
        with ThreadPoolExecutor(max_workers=2) as pool:
            old_future = pool.submit(calculate, args.litigate, old_tree, specs, triggers)
            new_future = pool.submit(calculate, args.litigate, ROOT, specs, triggers)
            old_result, new_result = old_future.result(), new_future.result()

        moved = []
        errors = []
        evaluated = 0
        for trigger in triggers:
            for deadline, spec in specs.items():
                evaluated += 1
                _, unit, direction, _, cite = spec
                old_due, old_nominal = old_result[(deadline, trigger)]
                new_due, new_nominal = new_result[(deadline, trigger)]
                delta = (new_due - old_due).days
                if direction == "before" and delta:
                    errors.append(f"{deadline} {trigger}: backward deadline moved {delta:+d} days")
                if delta < 0:
                    errors.append(f"{deadline} {trigger}: deadline moved earlier {delta} days")
                if delta:
                    # Business-day computations encounter state closures in
                    # the trigger-to-nominal interval. Calendar-day periods
                    # only encounter them in the last-day rollover interval.
                    if unit in ("business_days", "court_days"):
                        responsible = [h for h in HOLIDAYS if trigger < h <= new_nominal]
                    else:
                        responsible = [h for h in HOLIDAYS if old_nominal <= h < new_due]
                    old_roll = (old_due - old_nominal).days
                    new_roll = (new_due - new_nominal).days
                    roll_delta = new_roll - old_roll
                    if direction != "after":
                        errors.append(f"{deadline} {trigger}: non-forward deadline moved")
                    if not responsible:
                        errors.append(f"{deadline} {trigger}: moved without a Georgia-only holiday in its counting/rollover interval")
                    if unit not in ("business_days", "court_days") and new_nominal != old_nominal:
                        errors.append(f"{deadline} {trigger}: calendar-day nominal date changed")
                    if roll_delta and not any(old_nominal <= h < new_due for h in HOLIDAYS):
                        errors.append(f"{deadline} {trigger}: final-day roll changed {roll_delta} days without a Georgia holiday")
                    # The movement is the nominal skip plus the change in
                    # end-of-period roll. The latter can span a weekend or a
                    # federal holiday after a Georgia closure (for example,
                    # Good Friday 2026 rolls through Saturday/Sunday).
                    if delta != (new_nominal - old_nominal).days + roll_delta:
                        errors.append(f"{deadline} {trigger}: due shift does not equal nominal shift plus changed roll span")
                    relevant = sorted(set(responsible))
                    holiday = "; ".join(f"{h.isoformat()} {HOLIDAYS[h]}" for h in relevant)
                    moved.append((deadline, cite, direction, trigger, old_due, new_due, holiday))
        if errors:
            raise RuntimeError("invariant failed:\n" + "\n".join(errors[:30]))

        branch = subprocess.check_output(["git", "branch", "--show-current"],
                                        cwd=ROOT, text=True).strip()
        lines = [
            "# Georgia holiday effects on Bankr. N.D. Ga. deadlines", "",
            "Generated by scripts/georgia-forward-moves.py using litigate test's compiled deadline clock expressions.",
            f"Old source: {args.old_ref} (GitLab main). New source: this checkout branch {branch}.",
            "Trigger dates: every calendar date from 2026-01-01 through 2027-12-31.", "",
            f"Evaluated {evaluated} deadline/trigger pairs; {len(moved)} moved.", "",
            "Each listed date is a Georgia state holiday in the official 2026 schedule, under O.C.G.A. § 1-4-1(a)(2).",
            "The date delta is the engine's nominal count shift plus any change in the final-day roll span; that roll span can include intervening weekends or federal holidays.",
            "",
            "| Deadline id | Local rule cite | Direction | Trigger | Old due date | New due date | Georgia holiday skipped or rolled over (cite) |",
            "|---|---|---|---|---|---|---|",
        ]
        lines.extend(
            f"| {d} | {cite} | {direction} | {trigger} | {old} | {new} | {holiday} |"
            for d, cite, direction, trigger, old, new, holiday in moved)
        output = ROOT / "docs/solutions/georgia-forward-moves.md"
        output.write_text("\n".join(lines) + "\n")
        print(f"wrote {output.relative_to(ROOT)}: {evaluated} evaluated, {len(moved)} moved")
        return 0
    except (OSError, subprocess.CalledProcessError, RuntimeError, KeyError, ValueError) as exc:
        print(f"georgia-forward-moves: {exc}", file=sys.stderr)
        return 1
    finally:
        if old_tree:
            shutil.rmtree(old_tree, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
