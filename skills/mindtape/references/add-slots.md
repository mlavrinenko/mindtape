# `mt add` slot semantics

Reference for the mindtape skill. Read this before you fill slots on a real
template. It covers what `--describe` output does not show.

## Pass values as tokens, or as YAML on stdin

A token is `key=value` (set), `key+=value` (append to a list slot) or
`key@path` (read the value from a file). Pass a token after `--set` or bare
after `--`. Both parse the same:

```sh
mt add "Practice arpeggios" --set impact=8 --set tags+=technique
mt add "Practice arpeggios" -- impact=8 tags+=technique
mt add "Practice arpeggios" -- summary@notes.txt
```

- A bare positional is title text, never a token: `mt add "fix a=b parsing"`
  sets no slot.
- A dot in a key is part of the slot's name, not a path MindTape parses.
  `--set estimate.files=2` works only because the template wrote
  `slot("estimate.files")`.
- The first operator, `=` or `@`, separates key from value. So
  `summary=ping @ada` is prose holding an `@`, and `summary@v=1.txt` is a path
  holding one.

When you generate values instead of typing them, pass YAML with `-f -`. It is
one structured channel with no shell quoting, and a block scalar carries a
prose slot verbatim:

```sh
mt add -f - <<'YAML'
title: Practice arpeggios
estimate.files: 2
tags: [technique, daily]
summary: |
  Work the 7th chords hands-separate, then together at 60bpm.
YAML
```

## Use slot names as keys

A `--set` or `-f` key must name a slot of the active template. `mt add`
refuses an unknown key and writes nothing:
`mt add fix bug --set priority=3` fails with
`error: unknown slot: 'priority'` and a `hint:` to run `mt add --describe`.

- Run `mt add --describe`, with `-t <template>` for a non-default template,
  to list the slot names.
- Run with `--print` to see the result before it is written.
- An `e.g.` line under a slot is the template's example of a good value's
  shape. `mt add` never files it, so pass your own value.

A slot name need not match a field name. A template writes a framework
priority as `priority: ice(impact: slot("impact"), …)`, so its slots are the
inputs: `--set impact=8 --set confidence=0.9 --set ease=4`, never
`--set priority=8`. Under a strict framework (`framework` with
`allow-manual = false` in `[tasks.priority]`) a scalar priority is invalid, so
`mt add` splices the framework call into a template that omits priority, and
the inputs are still the slots. The same holds for any field the template
nests inside a call.

For tags, pass `+word` positionals, which fill the `tags` slot
(`mt add fix login +backend +auth`), or repeat the append operator:
`--set tags+=backend --set tags+=auth`. `mt add` refuses
`--set tags=backend,auth`: a `list` slot needs `+=`, or an array through `-f`.

## Write plain prose in a `text` slot

`mt add` escapes every `text` slot value as Typst markup before it writes the
file, whatever the source: `-f`, `--set`, `--edit`, a positional or the
environment, JSON or YAML. Escaping runs after parsing. A literal `` ` ``,
`*`, `_`, `#`, `[`, `]`, `@` or `<` lands as `` \` ``, `\*` and so on: safe,
never live Typst.

- Do not write code spans or emphasis in a `text` slot such as `summary` or
  `scope`, expecting them to render.
- When the body needs formatted text, such as a code span or a link, fill the
  slot with a placeholder, then edit the body by hand following
  `hand-editing.md`. The config checks only the section headers, not the prose
  under them.
- After `mt add`, search the written task for a backslash-backtick pair:
  `` rg -F '\`' <file> ``. A hit is almost always markup that landed escaped.

## A `raw` slot splices verbatim, by the template's choice

A template can declare a slot `kind: "raw"` instead of `kind: "text"`. Its
value splices into the body verbatim, code spans included:
`` `mt add` `` stays a code span, not `` \`mt add\` ``. Only the checked-in
template declares the kind. A `--set`, `-f` or environment value never skips
escaping because it looks like markup: the same value fills a `text` slot
escaped and a `raw` slot verbatim.

The value is pasted, not read. Declare `raw` for a slot that holds formatted
body content, such as `summary` or `body`.

- `mt add` parses a `raw` value as Typst markup and fails before writing
  anything, naming the slot, when it does not parse. That check covers a
  `#slot(…)` in the body, which is markup.
- It does not cover a `raw` slot in an argument position, where the value is
  an expression. Markup treats parens as plain text, so a broken expression
  reaches the file and `mt add` fails loading what it wrote. Use a typed kind
  wherever a value has a shape.

`mt add` refuses a slot with an unknown kind, such as `typst`, naming the slot
and listing the valid kinds. It never fills one as a quoted string.

## `key@path` reads a file and converts nothing

The `@` operator puts a file's bytes where a command-line value would go. The
slot's kind still decides what the bytes become:

```sh
mt add "Practice arpeggios" -- summary@notes.txt    # text slot: escaped
mt add "Split or re-cap ls.rs" -- remedy@remedy.typ # raw slot: verbatim
```

- Do not pass a `.md` file to a `text` slot. Its headings and code spans land
  as literal `#` and backticks, because nothing converts Markdown to Typst.
  Pass `.txt` for prose and `.typ` for a `raw` slot.
- Every other kind reads the file too: `weight@score.txt` fills an `int`. A
  file that does not fit fails naming the slot, with the error the same value
  typed inline gets.
- `mt add` strips trailing newlines and nothing else, so `key@path` equals
  `--set "key=$(cat path)"`.
- There is no `key+@path`, because a file is one value. Pass a list through
  `-f` as an array. There is no `key@-` either: `-f -` is the stdin channel,
  where a YAML block scalar carries a prose slot verbatim.

Use `@` when the file exists already, above all a `.typ` you wrote and
checked before filing the task. When you generate values in-process, use
`-f -`: one payload instead of one temp file per slot.
