#!/usr/bin/env python3
"""Report every changed leaf in lowered pack JSON between two build outputs."""

import argparse
import json
from pathlib import Path


def leaves(value, path=""):
    if isinstance(value, dict):
        for key, child in sorted(value.items()):
            yield from leaves(child, f"{path}/{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from leaves(child, f"{path}/{index}")
    else:
        yield path, value


def load(directory):
    return {p.name: json.loads(p.read_text()) for p in Path(directory).glob("*.json")}


def edge_rows(document):
    rows = {}
    for edge in document.get("edges", []):
        rows[edge["id"]] = edge
    for edge in document.get("links", []):
        rows[edge["id"]] = edge
    for instance, definition in document.get("instances", {}).items():
        for edge_id, edge in definition.get("patch_edges", {}).items():
            rows[f"{instance}/{edge_id}"] = edge
    return rows


def edge_leaves(document):
    result = {
        f"/{edge_id}/{path.lstrip('/')}" if path else f"/{edge_id}": value
        for edge_id, edge in edge_rows(document).items()
        for path, value in leaves(edge)
    }
    indexed = {str(index): edge["id"] for index, edge in enumerate(document.get("edges", []))}
    for hole in document.get("lowered", {}).get("holes", []):
        parts = hole.get("pointer", "").strip("/").split("/")
        if len(parts) >= 2 and parts[0] == "edges" and parts[1] in indexed:
            result[f"/{indexed[parts[1]]}/lowered.holes/{'/'.join(parts[2:])}"] = hole.get("reason")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", help="directory containing baseline lowered JSON")
    parser.add_argument("after", help="directory containing final lowered JSON")
    parser.add_argument("packs", nargs="+", help="pack JSON stems to compare")
    args = parser.parse_args()
    before, after = load(args.before), load(args.after)
    print("| Pack | Edge or pointer | Field | Old | New |")
    print("|---|---|---|---|---|")
    count = 0
    for stem in args.packs:
        filename = f"{stem}.json"
        if filename not in before or filename not in after:
            parser.error(f"missing {filename} in one of the build directories")
        left = edge_leaves(before[filename])
        right = edge_leaves(after[filename])
        for pointer in sorted(left.keys() | right.keys()):
            old, new = left.get(pointer), right.get(pointer)
            if old == new and pointer in left and pointer in right:
                continue
            parts = pointer.strip("/").split("/")
            edge = parts[0]
            field = "/".join(parts[1:]) if len(parts) > 1 else "<edge>"
            if pointer not in left:
                old_text = "<missing>"
            else:
                old_text = json.dumps(old, ensure_ascii=False, sort_keys=True)
            if pointer not in right:
                new_text = "<missing>"
            else:
                new_text = json.dumps(new, ensure_ascii=False, sort_keys=True)
            safe = lambda s: s.replace("|", "\\|").replace("`", "&#96;").replace("\n", " ")
            print(f"| {stem} | `{safe(edge)}` | `{safe(field)}` | `{safe(old_text)}` | `{safe(new_text)}` |")
            count += 1
    print(f"\nChanged lowered leaves: {count}")


if __name__ == "__main__":
    main()
