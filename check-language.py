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

"""Verify British spelling and English Spacing across a project.

British spelling with `-ise` endings, and English Spacing — two spaces after
`.`, `:`, `!` and `?` — apply to everything that is language:  prose,
documentation comments, string literals, identifiers and file names.  This
script checks the prose;  identifiers and file names are few enough to read.

Run `--self-test` first.  A checker that reports nothing is worthless unless it
has been shown to report something, so the self-test feeds it a known-bad and a
known-good fixture and fails loudly if either verdict is wrong.

Usage:
    check-language.py --self-test
    check-language.py FILE...
"""

import os
import re
import sys

# Spellings to reject.  Kept as patterns rather than a word list, because the
# `-ize`, `-ization` and `-yze` families are open-ended.
AMERICAN = (
    r"\b\w*ize[sdr]?\b",
    r"\b\w*izing\b",
    r"\b\w*ization[s]?\b",
    r"\b\w*yze[sd]?\b",
    r"\b\w*yzing\b",
    r"\blicens[ei]",
    r"\bcolor\b",
    r"\bbehavior",
    r"\bcenter\b",
    r"\bfulfill",
    r"\bartifact",
)

# Words the patterns above catch but which are correct British English.
INNOCENT = {
    "size", "sizes", "sized", "sizer", "prize", "prizes", "seize", "seizes",
    "maize", "resize", "resizes", "resized", "downsize", "capsize",
}

# Terms fixed by something outside the project:  a quoted interface label, or a
# name belonging to someone else.  Correcting these would break the reference
# rather than improve the prose.
FOREIGN = {"authorization"}

# `license` is American, but it is also a Cargo manifest key, another author's
# crate name, a field of `cargo_metadata`, part of a URL, and the conventional
# file name of the wider ecosystem.  It is therefore exempt only where it is
# plainly one of those, and flagged everywhere else — a blanket exemption would
# hide the word in genuine prose, which it did until 2026-08-22.
CODE_LICENSE = re.compile(
    r"(license[-_](file|id)|[.:\"`/]license\b|\blicense\s*=|dep:license"
    r"|LICENSE[-_.]|\bLICENSES?\b|/licenses/|`LICENSE'|UNLICENSE)"
)

# Abbreviations whose full stop does not end a sentence.
ABBREVIATION = re.compile(r"\b(e\.g|i\.e|cf|vs|etc|al|Mr|Dr|St)\.$")

# The character after the space is deliberately *not* constrained to a letter.
# An assertion message ending `...file: {problems:?}` is prose, and requiring a
# letter there let a whole class of violations through unnoticed.
SENTENCE_END = re.compile(r"(?<![0-9.])([.!?:]) (?![ ])")

# Text between backticks is code — an identifier, a file name, a command — and
# is governed by the language of whatever it names, not by this project's
# prose conventions.
CODE_SPAN = re.compile(r"`[^`]*`")

# A web address is somebody else's name for something.  It is quoted, never
# corrected, however it happens to be spelled.
URL = re.compile(r"<?https?://\S+>?")

# `License` inside a proper name — the title of "Apache License 2.0", or of
# the "GNU General Public License" — is that licence's own name, and
# `LicenseRef` is an SPDX keyword.  Neither is this project's to correct.
# A string literal that is Rust being *emitted* rather than English being
# printed.  A code generator's output is governed by Rust's rules, not these.
CODE_LITERAL = re.compile(r"(&\[|::|include_str!|\{\{|\}\}|\bpub |\bfn )")

PROPER_NAME = re.compile(
    r"(?:[A-Z][A-Za-z0-9.-]*\s+License|Licen[sc]eRef|DocumentRef)"
)


def prose(path, number, line, state):
    """Return the prose fragments of one line, or nothing if it carries none.

    Code is not prose:  a type annotation is allowed one space after its colon,
    and a method chain is allowed one after its full stop.  Only comments,
    documentation and substantial string literals are considered.
    """
    extension = os.path.splitext(path)[1]

    # The GPL notice is a verbatim quotation and must never be reworded.  It is
    # recognised by its own text rather than by line number:  a positional rule
    # would silently swallow real prose in any file whose header is shorter or
    # absent, which is precisely the failure the self-test exposed.
    if "GNU General Public License 3.0" in line and re.match(r"\s*[/#]", line):
        state["notice"] = True
        return []

    if state["notice"]:
        if re.match(r"^\s*(\\\*+/|#{20,}\s*)$", line):
            state["notice"] = False

        return []

    # The file information block references the LICENSE file by its real name.
    if "LICENSE" in line and "NOTE" in line:
        return []

    # Horizontal rules carry no language.
    if re.match(r"^\s*(#{20,}|/\*+/?|\|.*\||\\\*+/)\s*$", line):
        return []

    if extension == ".rs":
        comment = re.match(r"\s*(///|//!|//)\s?(.*)$", line)

        if comment:
            body = comment.group(2)

            # A fenced block inside a documentation comment is an example
            # written in Rust, and Rust is not governed by these rules.
            if body.strip().startswith("```"):
                state["code"] = not state["code"]
                return []

            return [] if state["code"] else [body]

        return [
            literal
            for literal in re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', line)
            if len(literal) > 12 and " " in literal
        ]

    # Hash comments are prose wherever they occur.  A Python docstring is
    # prose too, but the fixtures below are deliberately incorrect string
    # literals, and no rule separating the two survived contact with them.
    if extension in (".py", ".toml", ".yml") or os.path.basename(
        path
    ) == ".gitignore":
        comment = re.match(r"\s*#+\s?(.*)$", line)
        return [comment.group(1)] if comment else []

    if extension == ".md":
        if line.strip().startswith("```"):
            state["code"] = not state["code"]
            return []

        # Fenced code, indented code and table rows are not prose.
        if (
            state["code"]
            or line.startswith("    ")
            or line.lstrip().startswith("|")
        ):
            return []

    return [line]


def inspect(path):
    """Yield every finding in one file."""
    state = {"code": False, "notice": False}

    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, raw in enumerate(handle, 1):
            line = raw.rstrip("\n")

            for text in prose(path, number, line, state):
                if CODE_LITERAL.search(text):
                    continue

                text = URL.sub(" ", CODE_SPAN.sub(" ", text)).strip()

                if not text:
                    continue

                for pattern in AMERICAN:
                    for hit in re.finditer(pattern, text, re.IGNORECASE):
                        word = hit.group(0).lower()

                        if word in INNOCENT or word in FOREIGN:
                            continue

                        if PROPER_NAME.search(
                            text[max(0, hit.start() - 24):hit.end() + 10]
                        ):
                            continue

                        if word.startswith("licens") and CODE_LICENSE.search(
                            text[max(0, hit.start() - 12):hit.end() + 8]
                        ):
                            continue

                        yield number, "SPELLING", hit.group(0), text

                for hit in SENTENCE_END.finditer(text):
                    before = text[max(0, hit.start() - 5):hit.start() + 1]

                    if ABBREVIATION.search(before):
                        continue

                    yield number, "SPACING", hit.group(1), text


def report(paths):
    """Print every finding and return how many there were."""
    total = 0

    for path in paths:
        for number, kind, token, text in inspect(path):
            total += 1
            print(f"{path}:{number} [{kind}] {token!r}")
            print(f"    {text[:96]}")

    print(f"--- {total} findings in {len(paths)} files ---")

    return total


BAD = """\
//! This is a doc comment. It has one space after the full stop.
//! It normalizes things and mentions color and behavior.
//! A colon: one space.  A question? One space.
"""

GOOD = """\
//! This is a doc comment.  It has two spaces after the full stop.
//! It normalises things and mentions colour and behaviour.
//! A colon:  two spaces.  A question?  Two spaces.
"""


def self_test():
    """Prove the checker reports what it should, and only that.

    Returns the process exit status:  zero when the checker behaves.
    """
    import tempfile

    expected_bad = 6
    failures = []

    with tempfile.TemporaryDirectory() as directory:
        for name, content, wanted in (
            ("bad.rs", BAD, expected_bad),
            ("good.rs", GOOD, 0),
        ):
            path = os.path.join(directory, name)

            with open(path, "w", encoding="utf-8") as handle:
                handle.write(content)

            found = sum(1 for _ in inspect(path))

            if found != wanted:
                failures.append(
                    f"{name}:  expected {wanted} findings, got {found}"
                )
            else:
                print(f"self-test {name}:  {found} findings, as expected")

    for failure in failures:
        print(f"SELF-TEST FAILURE:  {failure}")

    return 1 if failures else 0


def main():
    """Dispatch on the command line."""
    arguments = sys.argv[1:]

    if not arguments:
        print(__doc__)
        return 2

    if arguments[0] == "--self-test":
        return self_test()

    return 1 if report(arguments) else 0


if __name__ == "__main__":
    sys.exit(main())

################################################################################
