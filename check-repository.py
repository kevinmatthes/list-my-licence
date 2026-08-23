#!/usr/bin/env python3

######################## GNU General Public License 3.0 ########################
##                                                                            ##
## Copyright (C) 2026 Kevin Matthes                                           ##
##                                                                            ##
## This program is free software: you can redistribute it and/or modify       ##
## it under the terms of the GNU General Public License as published by       ##
## the Free Software Foundation, either version 3 of the License, or          ##
## (at your option) any later version.                                        ##
##                                                                            ##
## This program is distributed in the hope that it will be useful,            ##
## but WITHOUT ANY WARRANTY; without even the implied warranty of             ##
## MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the              ##
## GNU General Public License for more details.                               ##
##                                                                            ##
## You should have received a copy of the GNU General Public License          ##
## along with this program.  If not, see <https://www.gnu.org/licenses/>.     ##
##                                                                            ##
################################################################################

"""Check line width and licence headers across a repository.

Two conventions that `rustfmt` and `clippy` cannot enforce between them.

**Width.**  Eighty characters, comments and documentation included.  `rustfmt`
holds the line for code, but it does not rewrap comments on stable, so prose
has to be checked separately.  Measured in **characters**:  `awk`'s `length`
counts bytes, which over-reports every line carrying an em dash.

**Headers.**  Every hand-written file carries the GPL notice.  Generated files
and the licence itself do not, and are named below rather than guessed at.

Usage:
    check-repository.py            # every tracked file
    check-repository.py FILE...
"""

import os
import re
import subprocess
import sys

WIDTH = 80

# Files that carry no header:  generated, or the notice itself.
EXEMPT = {"Cargo.lock", "LICENCE", "LICENSE", "README.md"}

# Only these are expected to carry one at all;  a fixture or datum is not.
HEADED = (".gitignore", ".py", ".rs", ".toml", ".yml")

NOTICE = "GNU General Public License 3.0"


def tracked():
    """Every file Git knows about, staged additions included."""
    listing = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard"],
        capture_output=True,
        text=True,
        check=True,
    )

    return [path for path in listing.stdout.split("\n") if path]


def width(path):
    """Yield (line number, length) for every line that is too wide."""
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip("\n")

            if len(line) > WIDTH:
                yield number, len(line)


def headed(path):
    """Whether the file opens with the licence notice.

    A script's shebang has to be the very first line of it, so the notice
    follows rather than opens;  the blank line between them is stepped over
    with it.
    """
    with open(path, encoding="utf-8", errors="replace") as handle:
        line = handle.readline()

        while line.startswith("#!") or (line.strip() == "" and line):
            line = handle.readline()

        return NOTICE in line


def wanted(path):
    """Whether this file is expected to carry a header."""
    name = os.path.basename(path)

    return name not in EXEMPT and (
        path.endswith(HEADED) or name in (".gitignore", ".clippy.toml")
    )


def main():
    """Report every violation and return a process status."""
    paths = sys.argv[1:] or tracked()
    findings = 0

    for path in paths:
        if os.path.basename(path) in EXEMPT or not os.path.isfile(path):
            continue

        for number, length in width(path):
            findings += 1
            print(f"{path}:{number}  {length} characters, limit {WIDTH}")

        if wanted(path) and not headed(path):
            findings += 1
            print(f"{path}  carries no licence header")

    print(f"--- {findings} findings in {len(paths)} files ---")

    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())

################################################################################
