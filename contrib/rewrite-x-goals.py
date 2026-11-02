#!/usr/bin/env python3
"""Rewrite every long-form `x-goals` block in truth/tasks to `verdicts(...)`.

Run from the repo root:

    python3 contrib/rewrite-x-goals.py [--dry-run] [PATH ...]

Each `x-goals: (group: (goal: verdict, ...), ...)` becomes one
`verdicts(given, ..neutral-names)` call per group: helps/hurts entries keep
their source order in the given dict, neutral names keep their source order in
the positional list, and no name appears in both. Reasons are copied verbatim,
including nested brackets, backtick spans and line breaks.

The scanner knows the Typst constructs that can hide a bracket or a comma:
line and block comments (nested), quoted strings and raw blocks (one or more
backticks, including fenced examples). A block that does not match the expected
shape is an error; the script reports the file and writes nothing.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_-]*\Z")
CANDIDATE = re.compile(r"^([ \t]*)x-goals:[ \t]*", re.MULTILINE)


class RewriteError(Exception):
    """A block the scanner cannot rewrite without guessing."""


def line_of(source: str, offset: int) -> int:
    return source.count("\n", 0, offset) + 1


def skip_line_comment(source: str, offset: int) -> int:
    newline = source.find("\n", offset)
    return len(source) if newline < 0 else newline


def skip_block_comment(source: str, offset: int) -> int:
    depth = 1
    i = offset + 2
    while i < len(source):
        if source.startswith("/*", i):
            depth += 1
            i += 2
        elif source.startswith("*/", i):
            depth -= 1
            i += 2
            if depth == 0:
                return i
        else:
            i += 1
    raise RewriteError(f"unclosed block comment at offset {offset}")


def skip_raw(source: str, offset: int) -> int:
    """Skip a raw block opened by the backtick run at `offset`."""
    i = offset
    while i < len(source) and source[i] == "`":
        i += 1
    ticks = i - offset
    while i < len(source):
        if source[i] != "`":
            i += 1
            continue
        end = i
        while end < len(source) and source[end] == "`":
            end += 1
        if end - i >= ticks:
            return end
        i = end
    raise RewriteError(f"unclosed raw block at offset {offset}")


def skip_string(source: str, offset: int) -> int:
    i = offset + 1
    while i < len(source):
        char = source[i]
        if char == "\\":
            i += 2
            continue
        if char == '"':
            return i + 1
        i += 1
    raise RewriteError(f"unclosed string at offset {offset}")


def skip_at(source: str, offset: int) -> int | None:
    """Return the end of a comment, string or raw block at `offset`, else None."""
    char = source[offset]
    if source.startswith("//", offset):
        return skip_line_comment(source, offset)
    if source.startswith("/*", offset):
        return skip_block_comment(source, offset)
    if char == "`":
        return skip_raw(source, offset)
    if char == '"':
        return skip_string(source, offset)
    return None


def raw_span_end(raw_spans: list[tuple[int, int]], offset: int) -> int | None:
    for start, end in raw_spans:
        if start <= offset < end:
            return end
    return None


def odd_quotes_on_line(
    source: str, offset: int, raw_spans: list[tuple[int, int]]
) -> bool:
    """True when `offset` sits inside an odd run of quotes on its line."""
    line_start = source.rfind("\n", 0, offset) + 1
    quotes = 0
    i = line_start
    while i < offset:
        raw_end = raw_span_end(raw_spans, i)
        if raw_end is not None:
            i = raw_end
            continue
        if source[i] == "\\":
            i += 2
            continue
        if source[i] == '"':
            quotes += 1
        i += 1
    return quotes % 2 == 1


def masked_spans(source: str) -> list[tuple[int, int]]:
    """Spans that can hide a field line: comments and raw blocks.

    Raw spans are recorded as they open, so a quote inside one (a shell
    command in a code span) does not make a later real code span look quoted.
    """
    spans: list[tuple[int, int]] = []
    raw_spans: list[tuple[int, int]] = []
    i = 0
    while i < len(source):
        if source[i] == "\\":
            i += 2
            continue
        if source[i] == "`" and not odd_quotes_on_line(source, i, raw_spans):
            end = skip_raw(source, i)
            spans.append((i, end))
            raw_spans.append((i, end))
            i = end
            continue
        if source.startswith("//", i) and not odd_quotes_on_line(
            source, i, raw_spans
        ):
            end = skip_line_comment(source, i)
            spans.append((i, end))
            i = end
            continue
        if source.startswith("/*", i) and not odd_quotes_on_line(
            source, i, raw_spans
        ):
            end = skip_block_comment(source, i)
            spans.append((i, end))
            i = end
            continue
        i += 1
    return spans


def is_masked(spans: list[tuple[int, int]], offset: int) -> bool:
    for start, end in spans:
        if start > offset:
            return False
        if start <= offset < end:
            return True
    return False


def match_bracket(source: str, offset: int) -> int:
    """Match a content bracket. Quotes and parens there are prose, not syntax."""
    depth = 0
    i = offset
    while i < len(source):
        char = source[i]
        if char == "[":
            depth += 1
            i += 1
            continue
        if char == "]":
            depth -= 1
            i += 1
            if depth == 0:
                return i
            continue
        if char == "\\":
            i += 2
            continue
        if source.startswith("//", i) or source.startswith("/*", i) or char == "`":
            i = skip_at(source, i)
            continue
        i += 1
    raise RewriteError(f"unclosed '[' at offset {offset}")


def match_delimiter(source: str, offset: int, opener: str, closer: str) -> int:
    """Return the offset just past `closer`, skipping nested constructs.

    A content bracket is skipped whole while matching a code delimiter,
    because the parens and braces inside it are prose. The reverse also holds:
    parens, quotes and braces in a reason never unbalance its bracket.
    """
    if opener == "[":
        return match_bracket(source, offset)
    nested = {"(": "[{", "{": "[("}[opener]
    pairs = {"(": ")", "[": "]", "{": "}"}
    depth = 0
    i = offset
    while i < len(source):
        char = source[i]
        if char == opener:
            depth += 1
            i += 1
            continue
        if char == closer:
            depth -= 1
            i += 1
            if depth == 0:
                return i
            continue
        skipped = skip_at(source, i)
        if skipped is not None:
            i = skipped
            continue
        if char in nested:
            i = match_delimiter(source, i, char, pairs[char])
            continue
        i += 1
    raise RewriteError(f"unclosed {opener!r} at offset {offset}")


def split_top_level(source: str, start: int, end: int) -> list[tuple[int, int]]:
    """Split [start, end) on commas that are not nested or inside a literal."""
    parts: list[tuple[int, int]] = []
    part_start = start
    i = start
    while i < end:
        char = source[i]
        if char == ",":
            parts.append((part_start, i))
            part_start = i + 1
            i += 1
            continue
        skipped = skip_at(source, i)
        if skipped is not None:
            if source.startswith("/*", i) or source.startswith("//", i):
                raise RewriteError(
                    f"comment inside an x-goals block at line "
                    f"{line_of(source, i)}; refusing to drop it"
                )
            i = skipped
            continue
        if char in "([{":
            pairs = {"(": ")", "[": "]", "{": "}"}
            i = match_delimiter(source, i, char, pairs[char])
            continue
        i += 1
    parts.append((part_start, end))
    return parts


def split_key_value(source: str, start: int, end: int) -> tuple[str, int, int]:
    """Return (key, value_start, value_end) for one `key: value` part."""
    i = start
    while i < end:
        char = source[i]
        if char == ":":
            return source[start:i].strip(), i + 1, end
        skipped = skip_at(source, i)
        if skipped is not None:
            if source.startswith("/*", i) or source.startswith("//", i):
                raise RewriteError(
                    f"comment in a key at line {line_of(source, i)}; "
                    f"refusing to drop it"
                )
            i = skipped
            continue
        if char in "([{":
            pairs = {"(": ")", "[": "]", "{": "}"}
            i = match_delimiter(source, i, char, pairs[char])
            continue
        i += 1
    raise RewriteError(
        f"expected `key: value` at line {line_of(source, start)}, got "
        f"{source[start:end]!r}"
    )


@dataclass
class Verdict:
    key: str
    effect: str
    reason: str | None


@dataclass
class Group:
    key: str
    given: list[Verdict] | None
    neutral: list[str] | None
    source: str | None = None


def parse_verdict(source: str, start: int, end: int) -> Verdict:
    while start < end and source[start].isspace():
        start += 1
    while end > start and source[end - 1].isspace():
        end -= 1
    match = re.match(r"(helps|hurts|neutral)\b", source[start:end])
    if not match:
        raise RewriteError(
            f"expected helps/hurts/neutral at line {line_of(source, start)}, "
            f"got {source[start:end]!r}"
        )
    effect = match.group(1)
    rest = start + match.end()
    while rest < end and source[rest].isspace():
        rest += 1
    reason = None
    if rest < end and source[rest] == "[":
        close = match_delimiter(source, rest, "[", "]")
        reason = source[rest + 1 : close - 1]
        rest = close
        while rest < end and source[rest].isspace():
            rest += 1
    if rest != end:
        raise RewriteError(
            f"trailing text after a {effect} verdict at line "
            f"{line_of(source, rest)}: {source[rest:end]!r}"
        )
    if effect == "neutral" and reason is not None:
        raise RewriteError(
            f"a neutral verdict takes no reason at line {line_of(source, start)}"
        )
    return Verdict("", effect, reason)


def parse_group(source: str, start: int, end: int) -> Group:
    while start < end and source[start].isspace():
        start += 1
    while end > start and source[end - 1].isspace():
        end -= 1
    value = source[start:end]
    if value.startswith("verdicts"):
        return Group("", None, None, source=value)
    if not value.startswith("("):
        raise RewriteError(
            f"expected a group dict or verdicts(...) at line "
            f"{line_of(source, start)}, got {value!r}"
        )
    close = match_delimiter(source, start, "(", ")")
    if source[close:end].strip():
        raise RewriteError(
            f"trailing text after a group dict at line {line_of(source, close)}"
        )
    given: list[Verdict] = []
    neutral: list[str] = []
    for part_start, part_end in split_top_level(source, start + 1, close - 1):
        if not source[part_start:part_end].strip():
            continue
        key, value_start, value_end = split_key_value(
            source, part_start, part_end
        )
        quoted = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', key)
        if quoted:
            try:
                name = json.loads(key)
            except json.JSONDecodeError as error:
                raise RewriteError(
                    f"bad quoted goal name {key!r} at line "
                    f"{line_of(source, part_start)}"
                ) from error
        elif IDENT.match(key):
            name = key
        else:
            raise RewriteError(
                f"bad goal name {key!r} at line {line_of(source, part_start)}"
            )
        verdict = parse_verdict(source, value_start, value_end)
        verdict.key = name
        if verdict.effect == "neutral":
            neutral.append(name)
        else:
            given.append(verdict)
    return Group("", given, neutral)


def parse_x_goals(source: str, start: int) -> tuple[list[Group], int]:
    close = match_delimiter(source, start, "(", ")")
    groups: list[Group] = []
    for part_start, part_end in split_top_level(source, start + 1, close - 1):
        if not source[part_start:part_end].strip():
            continue
        key, value_start, value_end = split_key_value(
            source, part_start, part_end
        )
        if key not in ("product", "process"):
            raise RewriteError(
                f"unknown x-goals group {key!r} at line "
                f"{line_of(source, part_start)}"
            )
        group = parse_group(source, value_start, value_end)
        group.key = key
        groups.append(group)
    names = [group.key for group in groups]
    if names != ["product", "process"]:
        raise RewriteError(
            f"expected product and process groups, got {names} at line "
            f"{line_of(source, start)}"
        )
    return groups, close


def render_group(group: Group, indent: str) -> list[str]:
    if group.source is not None:
        return [f"{indent}{group.key}: {group.source.strip()},"]
    lines = [f"{indent}{group.key}: verdicts("]
    if group.given:
        lines.append(f"{indent}  (")
        for verdict in group.given:
            reason = "" if verdict.reason is None else f"[{verdict.reason}]"
            lines.append(
                f"{indent}    {verdict.key}: {verdict.effect}{reason},"
            )
        lines.append(f"{indent}  ),")
    else:
        lines.append(f"{indent}  (:),")
    for name in group.neutral or []:
        lines.append(f"{indent}  {json.dumps(name, ensure_ascii=False)},")
    lines.append(f"{indent}),")
    return lines


def render_block(groups: list[Group], base: str) -> str:
    lines = [f"{base}x-goals: ("]
    for group in groups:
        lines.extend(render_group(group, base + "  "))
    lines.append(f"{base})")
    return "\n".join(lines)


def rewrite(source: str) -> tuple[str, int]:
    spans = masked_spans(source)
    edits: list[tuple[int, int, str]] = []
    for match in CANDIDATE.finditer(source):
        if is_masked(spans, match.start()):
            continue
        value_start = match.end()
        if value_start >= len(source) or source[value_start] != "(":
            raise RewriteError(
                f"x-goals at line {line_of(source, match.start())} does not "
                f"start with `(`"
            )
        groups, close = parse_x_goals(source, value_start)
        if all(group.source is not None for group in groups):
            continue
        base = match.group(1)
        edits.append((match.start(), close, render_block(groups, base)))
    if not edits:
        return source, 0
    parts: list[str] = []
    cursor = 0
    for start, end, replacement in edits:
        parts.append(source[cursor:start])
        parts.append(replacement)
        cursor = end
    parts.append(source[cursor:])
    return "".join(parts), len(edits)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "paths", nargs="*", type=Path, default=[Path("truth/tasks")]
    )
    args = parser.parse_args(argv)

    files: list[Path] = []
    for path in args.paths:
        if path.is_dir():
            files.extend(sorted(path.rglob("*.typ")))
        else:
            files.append(path)

    rewritten = 0
    changed = 0
    updates: list[tuple[Path, str]] = []
    for path in files:
        source = path.read_text(encoding="utf-8")
        try:
            updated, count = rewrite(source)
        except RewriteError as error:
            print(f"{path}: {error}", file=sys.stderr)
            return 1
        if count:
            changed += 1
            rewritten += count
            updates.append((path, updated))
    if not args.dry_run:
        for path, updated in updates:
            path.write_text(updated, encoding="utf-8")
    verb = "would rewrite" if args.dry_run else "rewrote"
    print(f"{verb} {rewritten} x-goals blocks in {changed} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
