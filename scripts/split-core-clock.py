#!/usr/bin/env python3
"""Split the compiled core clock library along its declaring .lit modules."""
import hashlib
import json
import re
import shutil
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
combined = Path(sys.argv[1])
source_root = root / "lit/core"
out = root / "crates/litigate-engine/clock/core"

def sha(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()

owners = {}
module_sources = {}
for path in sorted(source_root.rglob("*.lit")):
    text = path.read_text()
    match = re.search(r"(?m)^module\s+([\w.-]+)", text)
    if not match:
        continue
    module = match.group(1)
    module_sources[module] = (path, text)
    for decl in re.finditer(r"(?m)^(calendar|counting)\s+([\w.-]+)", text):
        kind, ident = decl.groups()
        key = ("calendars" if kind == "calendar" else "countingRules", ident)
        if key in owners and owners[key] != module:
            raise SystemExit(f"duplicate core clock owner for {key}: {owners[key]}, {module}")
        owners[key] = module

doc = json.loads(combined.read_text())
groups = {name: {"calendars": [], "countingRules": []} for name in module_sources}
for field in ("calendars", "countingRules"):
    for item in doc.get(field, []):
        module = owners.get((field, item["id"]))
        if module is None:
            raise SystemExit(f"no lit/core module owns {field} {item['id']}")
        groups[module][field].append(item)

shutil.rmtree(out, ignore_errors=True)
out.mkdir(parents=True)
base = doc["lowered"]
as_of = base["asOf"]
for module, contents in groups.items():
    path, text = module_sources[module]
    pin = {"name": module, "version": "1.0.0", "digest": sha(text.encode())}
    lowered = {"asOf": as_of, "pins": [pin]}
    world = json.dumps(lowered, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    lowered["worldDigest"] = sha(world)
    contents["schemaVersion"] = doc["schemaVersion"]
    contents["lowered"] = lowered
    name = module + ".json"
    target = out / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(contents, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
