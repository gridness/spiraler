#!/usr/bin/env python3
"""Generate tap definitions using checksums of the published release bytes."""

import argparse
import hashlib
import re
from pathlib import Path


def version_parts(version):
    # ASVS 2.2.1: versions used in paths and Ruby must be numeric semver.
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError("Version must be major.minor.patch")
    return tuple(map(int, version.split(".")))


def update_tap(version, repo, published, tap):
    requested = version_parts(version)
    # ASVS 1.2.5: allow only a GitHub repository slug in generated Ruby URLs.
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise ValueError("Repository must be owner/name")
    definitions = {
        "Casks/spiraler.rb": "spiraler-cask.rb",
        "Formula/spiraler.rb": "spiraler-formula.rb",
    }
    for relative in definitions:
        path = tap / relative
        if path.exists():
            match = re.search(r'^\s+version "([^"\n]+)"$', path.read_text(), re.M)
            if not match:
                raise ValueError(f"Cannot determine current version in {path}")
            if version_parts(match[1]) > requested:
                print("A newer Homebrew package is already published; skipping this older run.")
                return

    values = {"VERSION": version, "REPO": repo}
    for key, suffix in {
        "MACOS_SHA256": "aarch64.dmg",
        "ARM64_SHA256": "aarch64.AppImage",
        "X86_64_SHA256": "x86_64.AppImage",
    }.items():
        # ASVS 11.4.1: hash the downloaded release, never a potentially different rebuild.
        with (published / f"Spiraler_{version}_{suffix}").open("rb") as asset:
            digest = hashlib.sha256()
            for chunk in iter(lambda: asset.read(1024 * 1024), b""):
                digest.update(chunk)
            values[key] = digest.hexdigest()

    templates = Path(__file__).parent / "homebrew"
    for relative, template in definitions.items():
        content = (templates / template).read_text()
        for key, value in values.items():
            content = content.replace(f"@{key}@", value)
        if re.search(r"@[A-Z0-9_]+@", content):
            raise ValueError(f"Unresolved placeholder in {template}")
        destination = tap / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content)
        print(f"Updated {relative} to {version}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version")
    parser.add_argument("repo")
    parser.add_argument("published", type=Path)
    parser.add_argument("tap", type=Path)
    args = parser.parse_args()
    update_tap(args.version, args.repo, args.published, args.tap)
