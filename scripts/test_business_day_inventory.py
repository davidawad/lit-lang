"""Targeted regression checks for the generated business-day inventory."""
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "business_day_inventory", ROOT / "scripts/business-day-inventory.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def test_counts_and_flagged_summary_reconcile():
    MODULE.main()
    report = MODULE.OUT.read_text()
    assert "## Authored uses (50)" in report
    assert "## Generated clock appendix (97 counting rules)" in report
    summary = report.split("### Flagged uses", 1)[1].split("## Authored uses", 1)[0]
    authored = report.split("## Authored uses", 1)[1].split("## Generated clock appendix", 1)[0]
    summary_rows = [line for line in summary.splitlines() if line.startswith("| ") and "`lit/" in line]
    flagged_rows = [line for line in authored.splitlines() if "| FLAG:" in line]
    assert len(summary_rows) == len(flagged_rows) == 17
    assert "UNVERIFIED" not in report


def test_generation_is_reproducible():
    MODULE.main()
    first = MODULE.OUT.read_bytes()
    MODULE.main()
    assert MODULE.OUT.read_bytes() == first


if __name__ == "__main__":
    test_counts_and_flagged_summary_reconcile()
    test_generation_is_reproducible()
