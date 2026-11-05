#!/usr/bin/env python3
"""Regenerate docs/solutions/business-day-inventory.md from lit/ and clock IR."""
from pathlib import Path
import json
import re

ROOT = Path(__file__).resolve().parents[1]
CLOCK_DIR = ROOT / "crates/litigate-engine/clock/core"
OUT = ROOT / "docs/solutions/business-day-inventory.md"

# Public source files searched for a court-specific definition. Keep this
# inventory explicit so a negative result can be reproduced.
COURT_SOURCE = {
    "akd-criminal": ("sources/fed-dak/",),
    "bankr-gan": ("sources/bankr-gan/ganb-lbr-2019-09-01-sectioned.txt",),
    "bankr-mie": ("sources/bankr-mie/mie-local-rules-2018-12-10-sectioned.txt",),
    "bankr-okw": ("sources/bankr-okw/okwb-lbr-2026-01-15-sectioned.txt",),
    "ca7": ("sources/ca7/ca7-rules-and-ops-2025-12-01.txt",),
    "caaf": ("sources/caaf/caaf-rules.txt",),
    "gasd-criminal": ("sources/sd-ga/sdga-local-rules-sectioned.txt",),
    "gud-criminal": ("sources/fed-dguam/guam-criminal-local-rules-2020-06-24-sectioned.txt",),
    "hid-criminal": ("sources/fed-dhaw/dhaw-criminal-local-rules-2023-01-01-sectioned.txt",),
    "med": ("sources/dme/local-rules-sectioned.txt",), "med-criminal": ("sources/dme/local-rules-sectioned.txt",),
    "miwd-criminal": ("sources/miwd/miwd-local-criminal-rules-sectioned.txt",),
    "nvd-criminal": ("lit/nvd-criminal/local-rules.lit",),
    "ohnd-criminal": ("sources/ohnd/ohnd-local-criminal-rules-2024-10-01-sectioned.txt",),
    "vid-criminal": ("sources/vid/vid-local-criminal-rules-2026-02-01-sectioned.txt",),
    "core": ("lit/core/frap26.lit", "lit/core/frcp6.lit", "lit/core/frcrimp45.lit", "lit/core/itc210.lit", "lit/core/rcfc6.lit"),
}
SPECIFIC_DEFINITIONS = {
    "bankr-gan": ('“Business Days” means every day, excluding Saturdays, Sundays, and legal holidays.', "N.D. Ga. BLR 9001-1(f)", "sources/bankr-gan/ganb-lbr-2019-09-01-sectioned.txt"),
    "nvd-criminal": ('“business day” means any day that is not a Saturday, Sunday, legal holiday or day the Chief Judge closed the courthouse.', "D. Nev. LCR 32-1(e)", "lit/nvd-criminal/local-rules.lit"),
}
JURISDICTION = {
    "akd-criminal": "Alaska — U.S. District Court, District of Alaska",
    "bankr-gan": "Georgia — U.S. Bankruptcy Court, Northern District of Georgia",
    "bankr-mie": "Michigan — U.S. Bankruptcy Court, Eastern District of Michigan",
    "bankr-okw": "Oklahoma — U.S. Bankruptcy Court, Western District of Oklahoma",
    "ca7": "Illinois, Indiana, Wisconsin — U.S. Court of Appeals, Seventh Circuit",
    "caaf": "Federal — U.S. Court of Appeals for the Armed Forces",
    "gasd-criminal": "Georgia — U.S. District Court, Southern District of Georgia",
    "gud-criminal": "Guam — U.S. District Court, District of Guam",
    "hid-criminal": "Hawaii — U.S. District Court, District of Hawaii",
    "med": "Maine — U.S. District Court, District of Maine",
    "med-criminal": "Maine — U.S. District Court, District of Maine",
    "miwd-criminal": "Michigan — U.S. District Court, Western District of Michigan",
    "nvd-criminal": "Nevada — U.S. District Court, District of Nevada",
    "ohnd-criminal": "Ohio — U.S. District Court, Northern District of Ohio",
    "vid-criminal": "U.S. Virgin Islands — U.S. District Court, District of the Virgin Islands",
    "core": "Federal / nationwide rule",
}

def audit_definition(forum, rule_id=""):
    if forum in SPECIFIC_DEFINITIONS:
        quote, cite, path = SPECIFIC_DEFINITIONS[forum]
        return f'"{quote}" ([{cite}]({path}))'
    if forum == "core":
        federal = {
            "frap26": ("FRAP 26(a)(6): ‘legal holiday’ includes a day declared a holiday by the state where the district court is located.", "https://www.law.cornell.edu/rules/frap/rule_26"),
            "frcp6": ("FRCP 6(a)(6)(C): ‘legal holiday’ includes any other day declared a holiday by the state where the district court is located.", "https://www.law.cornell.edu/rules/frcp/rule_6"),
            "frcrimp45": ("FRCrimP 45(a)(6)(C): ‘legal holiday’ includes any other day declared a holiday by the state where the district court is located.", "https://www.law.cornell.edu/rules/frcrimp/rule_45"),
            "itc210": ("19 C.F.R. § 201.14(a): the rule identifies Saturdays, Sundays, and Federal legal holidays; it does not define ‘business day’.", "https://www.ecfr.gov/current/title-19/section-201.14"),
            "rcfc6": ("RCFC 6(a): uses ‘legal holiday’ for its deadline computation; it does not define ‘business day’.", "https://www.uscourts.gov/rules-policies/current-rules-practice-procedure/federal-claims-rules"),
        }
        quote, url = federal[rule_id]
        return f'"{quote}" ([governing rule]({url})); no separate definition of “business day”'
    searched = []
    for item in COURT_SOURCE.get(forum, ()):
        path = ROOT / item
        if path.is_dir():
            searched.extend(p.relative_to(ROOT).as_posix() for p in sorted(path.glob("*.txt")))
        elif path.is_file():
            searched.append(item)
    if not searched:
        return "audit incomplete: no vendored rule text available; search was limited to the cited declaration"
    links = ", ".join(f"[`{p}`]({p})" for p in searched)
    return f"none: not defined in searched source text ({links})"


def main():
    rules = {}
    for path in sorted(CLOCK_DIR.glob("*.json")):
        clock = json.loads(path.read_text())
        rules.update({r["id"]: r for r in clock["countingRules"]})
    rows = []
    for base in (ROOT / "lit", ROOT / "cases"):
        for path in sorted(list(base.rglob("*.lit")) + list(base.rglob("*.case"))):
            lines = path.read_text().splitlines()
            for number, line in enumerate(lines, 1):
                if "business_days" not in line:
                    continue
                rule_match = re.search(r"counted_by\s+([\w-]+)", line)
                if rule_match:
                    rule_id = rule_match.group(1)
                    deadline = re.search(r"deadline\s+([\w-]+)", line)
                    cite = re.search(r'cite\s+"([^"]+)"', line)
                    # Delegated declarations carry the governing cite in their body.
                    if not cite:
                        for following in lines[number:min(number + 8, len(lines))]:
                            cite = re.search(r'cite\s+"([^"]+)"', following)
                            if cite:
                                break
                    kind = "deadline"
                    item = deadline.group(1) if deadline else "(unnamed)"
                    citation = cite.group(1) if cite else "UNVERIFIED: no inline cite"
                else:
                    # short_period clauses are the 18 shared rule definitions,
                    # not matter deadlines; rule id is the containing file stem.
                    declarations = [re.search(r"^counting\s+([\w-]+)", prior)
                                    for prior in lines[:number]]
                    declaration = next((m for m in reversed(declarations) if m), None)
                    rule_id = declaration.group(1) if declaration else path.stem
                    kind, item = "short_period", f"line {number}"
                    cite_match = None
                    if declaration:
                        for prior in reversed(lines[:number]):
                            if re.match(r"\s*counting\s+", prior):
                                cite_match = re.search(r'cite\s+"([^"]+)"', prior)
                                break
                    citation = (rules.get(rule_id, {}).get("label") or
                                (cite_match.group(1) if cite_match else "UNVERIFIED"))
                rule = rules.get(rule_id, {})
                holidays = rule.get("holidays", [])
                if not holidays:
                    # Rules outside the shared core clock (for example CAAF)
                    # are described by their counting declaration in source.
                    for prior in reversed(lines):
                        if prior.lstrip().startswith("holidays "):
                            holidays = [prior.strip().split()[1]]
                            break
                calendar = "; ".join(holidays) or "none authored"
                if rule.get("forwardHolidays"):
                    calendar += "; forward extension: " + ", ".join(rule["forwardHolidays"])
                weekdays = ", ".join(rule.get("closedWeekdays", [])) or "none authored"
                forum = path.parent.name
                rows.append((forum, path.relative_to(ROOT).as_posix(), number, kind,
                             item, citation, rule_id, rule.get("label") or citation,
                             weekdays, calendar, rule.get("sourceUrl", "")))

    rows.sort(key=lambda r: (r[0], r[1], r[2]))
    if len(rows) != 50:
        raise SystemExit(f"expected 50 authored business_days uses, found {len(rows)}")
    periods = clock["countingRules"]
    if len(periods) != 97:
        raise SystemExit(f"expected 97 generated counting periods, found {len(periods)}")

    flagged = [r for r in rows if (" after " in Path(r[1]).read_text().splitlines()[r[2]-1]
                                   or r[3] == "short_period") and
               ("district-state-holidays" in r[9] or
               (r[9].endswith("holidays") and "us-federal-holidays" in r[9]
                and "; forward extension:" in r[9]))]
    flagged_keys = {(r[1], r[2]) for r in flagged}
    out = [
        "# Business-day uses and pack calendars",
        "",
        "Generated by `scripts/business-day-inventory.py`; source is `lit/` plus `crates/litigate-engine/clock/core/*.json`.",
        "",
        "## Findings",
        "",
        f"**Flagged uses: {len(flagged)}.** These select a counting rule whose pack calendar closes federal holidays but either has an empty `district-state-holidays` placeholder or lists state holidays only for forward extension. FRCP 6(a)(6)(C), FRBP 9006(a)(6)(C), and FRCrimP 45(a)(6)(C) define a legal holiday to include a day declared a holiday by the state where the district court is located. The calendar's open-day set can therefore count an intermediate state holiday as open even though the governing computation rule treats it as a legal holiday. This flags a rule/calendar mismatch for PM triage; it does not establish which individual state holiday dates affect a particular deadline. The state holiday text and court-specific closure orders must be checked for each flagged court.",
        "",
        "### Flagged uses",
        "",
        "| State / court (group) | Source location | Deadline / use | Governing local rule cite | Counting rule | Calendar gap |",
        "|---|---|---|---|---|---|",
    ]
    for forum, file, line, kind, item, cite, rid, label, weekdays, holidays, url in flagged:
        out.append(f"| {JURISDICTION.get(forum, forum)} | `{file}:{line}` | {kind}: `{item}` | {cite} | `{rid}` ({label}) | State holiday excluded from forward counting only / absent from calendar open-day set |")
    out += [
        "",
        "## Authored uses (50)",
        "",
        "Rows are grouped by state and court; the shared `core` rules are labeled federal/nationwide. `short_period` rows are counting-rule definitions rather than deadlines and have no deadline id or district citation. Rule source links identify the public text underlying the pack rule. The pack weekday/holiday columns report the compiled counting rule; federal holiday names are represented by the `us-federal-holidays` calendar.",
        "",
        "| State / court (group) | Source location | Kind / deadline id | Governing rule cite | Counting rule | Pack calendar (closed weekdays; holidays) | Governing definition / comparison |",
        "|---|---|---|---|---|---|---|",
    ]
    for forum, file, line, kind, item, cite, rid, label, weekdays, holidays, url in rows:
        definition_cite = "FRCrimP 45(a)(6)(C)" if rid.startswith("frcrimp") else ("FRBP 9006(a)(6)(C)" if rid.startswith("frbp") else "FRCP 6(a)(6)(C)")
        rule_url = "https://www.law.cornell.edu/rules/frcp" if rid.startswith("frcp") else ("https://www.uscourts.gov/rules-policies/current-rules-practice-procedure/federal-rules-bankruptcy-procedure" if rid.startswith("frbp") else ("https://www.uscourts.gov/rules-policies/current-rules-practice-procedure/federal-rules-criminal-procedure" if rid.startswith("frcrimp") else ""))
        definition_cite = f"[{definition_cite}]({rule_url})" if rule_url else definition_cite
        definition = (f'FLAG: "any other day declared a holiday by the state where the district court is located" ({definition_cite}); pack state holiday is absent from intermediate-day calendar. Court definition audit: {audit_definition(forum, rid)}'
                     if ((file, line) in flagged_keys and ("district-state-holidays" in holidays or
                         ("; forward extension:" in holidays and "us-federal-holidays" in holidays))
                     ) else audit_definition(forum, rid))
        out.append(f"| {JURISDICTION.get(forum, forum)} | `{file}:{line}` | {kind}: `{item}` | {cite} | `{rid}` ({label}) | {weekdays}; {holidays} | {definition} |")
    out += ["", "## Generated clock appendix (97 counting rules)", "",
            "Each row is a compiled counting rule used by one or more generated business periods. `holiday sets` are the pack calendar inputs; `forward-only` sets affect forward deadline extension where authored.",
            "", "| Counting rule | Rule / source | Closed weekdays | Holiday sets | Forward-only holiday sets |", "|---|---|---|---|---|"]
    for rule in sorted(periods, key=lambda r: r["id"]):
        label = rule.get("label", "")
        url = rule.get("sourceUrl", "")
        source = f"[{label}]({url})" if url.startswith("http") else label
        out.append(f"| `{rule['id']}` | {source} | {', '.join(rule.get('closedWeekdays', [])) or 'none'} | {', '.join(rule.get('holidays', [])) or 'none'} | {', '.join(rule.get('forwardHolidays', [])) or 'none'} |")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
