#!/usr/bin/env python3
"""Update exactly one URL-adjacent checksum per Homebrew platform."""

import re
import sys
from pathlib import Path

PLATFORMS = ("macos-silicon", "macos-intel", "linux-amd64", "linux-arm64")
NUMERIC = r"(?:0|[1-9][0-9]*)"
PRERELEASE = r"(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
VERSION_PATTERN = re.compile(
    NUMERIC + r"\." + NUMERIC + r"\." + NUMERIC
    + r"(?:-" + PRERELEASE + r"(?:\." + PRERELEASE + r")*)?"
    + r"(?:\+[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*)?")


def render_formula(source, version, checksums):
    if not VERSION_PATTERN.fullmatch(version):
        raise ValueError("Invalid release version")
    if set(checksums) != set(PLATFORMS):
        raise ValueError("All four platform checksums are required")
    if any(not re.fullmatch(r"[a-f0-9]{64}", value) for value in checksums.values()):
        raise ValueError("Invalid SHA256 checksum")
    result, count = re.subn(r'(?m)^(\s*version )"[^"\n]+"$',
                            lambda m: m[1] + '"' + version + '"', source)
    if count != 1:
        raise ValueError("Expected exactly one formula version")
    for platform in PLATFORMS:
        pattern = (r'(?m)^(\s*url "[^"\n]*-' + re.escape(platform)
                   + r'"\n\s*sha256 )"[A-Za-z0-9_]+"$')
        result, count = re.subn(pattern,
                                lambda m: m[1] + '"' + checksums[platform] + '"',
                                result)
        if count != 1:
            raise ValueError("Expected exactly one URL/checksum for " + platform)
    return result


def main():
    if len(sys.argv) != 7:
        raise SystemExit("Usage: render-homebrew-formula.py FORMULA VERSION "
                         "MACOS_SILICON MACOS_INTEL LINUX_AMD64 LINUX_ARM64")
    path = Path(sys.argv[1])
    result = render_formula(path.read_text(), sys.argv[2],
                            dict(zip(PLATFORMS, sys.argv[3:])))
    # Validate the entire result before touching the formula.
    path.write_text(result)


if __name__ == "__main__":
    main()
