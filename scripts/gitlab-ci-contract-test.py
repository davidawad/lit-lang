#!/usr/bin/env python3
"""Check the GitLab pipeline keeps CI and tagged binary releases connected."""

from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[1]
PIPELINE = yaml.safe_load((ROOT / ".gitlab-ci.yml").read_text(encoding="utf-8"))


def main() -> None:
    jobs = PIPELINE
    assert PIPELINE["stages"] == ["verify", "tag", "binaries", "release"]
    assert jobs["ci"]["stage"] == "verify"
    assert jobs["ci"]["script"] == [
        "python3 scripts/gitlab-ci-contract-test.py",
        "just ci",
    ]
    assert jobs["linux-binaries"]["parallel"]["matrix"][0]["TARGET"] == "x86_64-unknown-linux-gnu"
    assert jobs["linux-binaries"]["parallel"]["matrix"][1]["TARGET"] == "x86_64-unknown-linux-musl"
    assert jobs["macos-binaries"]["parallel"]["matrix"][0]["TARGET"] == "aarch64-apple-darwin"
    assert jobs["macos-binaries"]["parallel"]["matrix"][1]["TARGET"] == "x86_64-apple-darwin"
    assert jobs["wasm-binaries"]["stage"] == "binaries"
    assert jobs["auto-release-tag"]["stage"] == "tag"
    assert jobs["auto-release-tag"]["resource_group"] == "release-tagging"
    assert "RELEASE_TAG_TOKEN" in jobs["auto-release-tag"]["script"][0]
    assert jobs["publish-release"]["needs"] == [
        {"job": "linux-binaries", "artifacts": True},
        {"job": "macos-binaries", "artifacts": True},
        {"job": "wasm-binaries", "artifacts": True},
    ]
    release_command = jobs["publish-release"]["script"][-1]
    assert "glab release create" in release_command
    assert "--use-package-registry" in release_command
    assert "dist/*.tar.gz" in release_command
    assert "dist/LITLOCK-ROOTS" in release_command
    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    assert "https://gitlab.com/davidawad/lit-lang/-/releases" in readme
    assert "GitHub Release" not in readme
    for path in (ROOT / ".github" / "workflows").glob("*.yml"):
        raise AssertionError(f"GitHub workflow remains active: {path.name}")
    print("GitLab CI contract: pass")


if __name__ == "__main__":
    main()
