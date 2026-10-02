# `mt add` slot semantics

Reference for the mindtape skill. Read this when filling slots on a real
template — the things that surprise callers, none of which you can infer from
`--describe` output alone.

## Values are data, and there is one token grammar

A value is `key=value` (set), `key+=value` (append to a list slot) or
`key@path` (read the value out of a file). Pass a token after `--set` or bare
after `--`; both parse identically:

```sh
mt add "Practice arpeggios" --set impact=8 --set tags+=technique
mt add "Practice arpeggios" -- impact=8 tags+=technique
mt add "Practice arpeggios" -- summary@notes.txt
```

A bare positional is title text, never a token — `mt add "fix a=b parsing"`
sets no slot. A dot in a key is part of the slot's name, not a path MindTape
parses: `--set estimate.files=2` works only because the template wrote
`slot("estimate.files")`. The first operator separates, `=` or `@` whichever
comes first, so `summary=ping @ada` is prose holding an `@` and
`summary@v=1.txt` is a path holding one.

If you are generating values rather than typing them, prefer `-f -` with
YAML. One structured channel, no shell quoting, and a block scalar carries a
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

## Token keys are slot names, and unknown ones are a hard error

A `--set`/`-f` key must name a slot of the active template. An unrecognised
key fails the command: `mt add fix bug --set priority=3` errors with
`unknown --set/-f key(s): priority; known slots: title, ...` rather than
writing anything. Run `mt add --describe` (plus `-t <template>` for a
non-default one) to see the real slot names, and `--print` to check the
result before it's written.

Slot names need not match field names. A framework priority is authored as
`priority: ice(impact: slot("impact"), …)`, so the slots are its leaves —
`--set impact=8 --set confidence=0.9 --set ease=4`, never `--set priority=8`.
Under a strict framework (`framework` + `allow-manual = false` in
`[tasks.priority]`) a scalar priority is invalid by construction, so even a
template that omits priority gets the framework call spliced in and the leaves
are still the slots. Same for any field the template nests inside a call.

Tags: `+word` positionals are sugar for the `tags` slot (`mt add fix login
+backend +auth`); or repeat the append operator, `--set tags+=backend --set
tags+=auth`. A bare `--set tags=backend,auth` is a hard error — a `list` slot
needs `+=`, or an array through `-f`.

## `text` slots are escaped, not markup

Every `text`-kind slot value (`-f`/`--set`/`--edit`/positional/environment,
JSON or YAML — the source makes no difference, escaping runs after parsing) is
markup-escaped before it lands in the file: a literal `` ` `` / `*` / `_` /
`#` / `[` / `]` / `@` / `<` you type becomes `` \` ``, `\*`, etc. — safe, but
never live Typst. So don't author code spans or emphasis inside slot content
(a `summary`/`scope` fill) expecting them to render — write plain prose there.
If a task genuinely needs formatted body text (a code span, a link), fill the
slot with a placeholder and hand-edit the body afterward — sanctioned per
"Authoring or editing a task file by hand" in SKILL.md, since only the section
*headers* are config-enforced, not the prose under them. Heuristic: after
`mt add`, search the written task for a literal backslash-backtick pair
(`` rg -F '\`' <file> ``) — a hit is almost always markup you meant literally,
not prose you meant escaped.

## `raw` slots opt out of escaping — a template's choice, never a caller's

A template can declare `kind: "raw"` instead of `kind: "text"` to skip the
escaping above: a filled value splices into the body verbatim, code spans and
all — `` `mt add` `` stays a code span, not `` \`mt add\` ``. This is strictly
a *template author*'s decision: only the checked-in template file declares the
kind, so a `--set`/`-f`/environment value never escapes escaping on its own
just by looking like markup — the same value fills a `text` slot escaped and a
`raw` slot verbatim, depending only on which the template declared.

The name is the contract: the value is pasted, not read. Prefer `raw` when a
slot's whole point is formatted body content (a `summary`/`body`). A value
that fails to parse as Typst markup fails `mt add` outright, naming the slot,
before anything is written — but that check reads the value as markup, which
is the plane a `#slot(…)` in the body lands on. A `raw` slot spliced into an
argument position is not covered: there the value is an expression, and
markup treats parens as ordinary text, so a broken one reaches the file and
`mt add` fails loading what it just wrote. Anywhere a value has a shape, use
a typed kind instead.

A slot with an unknown kind, such as `typst`, fails naming the slot and
listing the valid kinds; it never fills as a quoted string.

## `key@path` reads a file, and converts nothing

The `@` operator puts a file's bytes where a command-line value would go. It
carries; it does not convert. The slot's kind still decides what the bytes
become, exactly as above:

```sh
mt add "Practice arpeggios" -- summary@notes.txt    # text slot: escaped
mt add "Split or re-cap ls.rs" -- remedy@remedy.typ # raw slot: verbatim
```

Do not pass a `.md` file to a `text` slot expecting Markdown to render — the
headings and code spans arrive as the literal `#` and backticks they are, and
nothing anywhere converts Markdown to Typst. `.txt` for prose, `.typ` for a
`raw` slot.

Every other kind reads the file too (`weight@score.txt` fills an `int`), and a
file that does not fit fails naming the slot, the same error the same value
typed inline gets. Trailing newlines are stripped and nothing else is, so `@`
is a drop-in for `--set "key=$(cat path)"`. There is no `key+@path` — a file
is one value, and a list goes through `-f` as an array — and no `key@-`: `-f
-` is the stdin channel, where a YAML block scalar already carries a prose
slot verbatim.

Reach for `@` when the file already exists, above all a `.typ` you wrote and
checked before filing the task. When you are generating the values in-process,
`-f -` is still shorter: one payload instead of one temp file per slot.
