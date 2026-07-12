// Render www/generated/cli.json — the clap-reflected command tree (facts only,
// from `mt _cli --json`) — as the styled CLI reference. This file owns
// presentation; the JSON owns facts. Per-command / per-arg prose and examples
// come from cli-notes.typ (www-only) and are merged in by dotted key
// (`ls`, `ls.filter`, `config.show`, `config.show.origin`).
//
// Refs: truth/tasks/structured-cli-reference-from-command-dump.typ

#import "ui.typ": terminal, terminal-ansi
#import "cli-notes.typ": cli-notes

#let _dim(body) = text(fill: rgb("#5b6570"), size: 0.9em, body)

// ── argument signature, e.g. `-n, --limit <LIMIT>` or `[FILTER]` ──────────────
#let _sig(arg) = {
  if arg.kind == "positional" {
    let name = if arg.value_name != none { arg.value_name } else { upper(arg.id) }
    let s = if arg.required { "<" + name + ">" } else { "[" + name + "]" }
    return if arg.multiple { s + "…" } else { s }
  }
  let flags = ()
  if arg.short != none { flags.push("-" + arg.short) }
  if arg.long != none { flags.push("--" + arg.long) }
  let s = flags.join(", ")
  if arg.value_name != none { s = s + " <" + arg.value_name + ">" }
  if arg.multiple { s = s + "…" }
  s
}

// enum-valued args → a tight list of `value` — help
#let _pvals(arg) = {
  let items = arg.possible_values.map(pv => {
    if pv.about != none { [#raw(pv.name) — #pv.about] } else { raw(pv.name) }
  })
  list(..items)
}

// Command/arg description: the short `about`, then any long-help remainder,
// dimmed. clap's long help repeats the short line as its first paragraph, so
// strip that prefix to avoid echoing it.
#let _desc(short, long) = {
  if short != none [#short]
  if long != none {
    let extra = if short != none and long.starts-with(short) {
      long.slice(short.len()).trim()
    } else if long != short { long } else { "" }
    if extra != "" { parbreak(); _dim[#extra] }
  }
}

// ── one argument's description cell ──────────────────────────────────────────
#let _arg-desc(arg, key) = {
  _desc(arg.about, arg.long_about)
  if arg.possible_values != none { parbreak(); _pvals(arg) }
  let meta = ()
  if arg.default != none { meta.push("default: " + arg.default) }
  if arg.multiple { meta.push("repeatable") }
  if meta.len() > 0 { parbreak(); _dim(raw(meta.join("   ·   "))) }
  let note = cli-notes.at(key, default: none)
  if note != none {
    if "prose" in note { parbreak(); note.prose }
    if "examples" in note { note.examples }
  }
}

#let _args-table(args, key-prefix) = {
  if args.len() == 0 { return }
  table(
    columns: (auto, 1fr),
    align: (left + top, left + top),
    stroke: none,
    inset: (x: 6pt, y: 5pt),
    ..args.map(arg => (raw(_sig(arg)), _arg-desc(arg, key-prefix + "." + arg.id))).flatten()
  )
}

// ── generated sample-output captures attached to a command via its note ──────
#let _outputs(note) = {
  if note == none { return }
  for slug in note.at("outputs-ansi", default: ()) {
    terminal-ansi(read("../generated/" + slug + ".ansi"))
  }
  for slug in note.at("outputs", default: ()) {
    terminal(read("../generated/" + slug + ".txt"))
  }
}

// ── one command block (recursive over subcommands) ───────────────────────────
#let _command(cmd, path) = {
  let key = path.join(".")
  let note = cli-notes.at(key, default: none)
  heading(level: path.len() + 1, raw("mt " + path.join(" ")))
  _desc(cmd.about, cmd.long_about)
  if note != none and "prose" in note { parbreak(); note.prose }
  _args-table(cmd.args, key)
  if note != none and "examples" in note { note.examples }
  _outputs(note)
  for sub in cmd.subcommands { _command(sub, path + (sub.name,)) }
}

// ── public: globals, a named group, and the coverage gate ────────────────────
#let _by-name(data) = {
  let m = (:)
  for c in data.commands { m.insert(c.name, c) }
  m
}

#let _group-names(groups, title) = {
  for g in groups {
    if g.at(0) == title { return g.at(1) }
  }
  panic("cli-ref: unknown group " + title)
}

// The three global flags, rendered once (they repeat on every command in
// clap's own `--help`; the reference shows them a single time).
#let render-globals(data) = _args-table(data.globals, "@global")

// Render every command in the named group, in map order.
#let cli-group(data, groups, title) = {
  let by = _by-name(data)
  for name in _group-names(groups, title) { _command(by.at(name), (name,)) }
}

// Every dotted key a note may legally attach to (commands + their args, recursive).
#let _cmd-keys(cmd, path) = {
  let key = path.join(".")
  let keys = (key,)
  for a in cmd.args { keys.push(key + "." + a.id) }
  for sub in cmd.subcommands { keys += _cmd-keys(sub, path + (sub.name,)) }
  keys
}

// Two-way completeness gate — fail the build rather than silently drop a
// command or dangle a note. Call once per page.
#let cli-gate(data, groups) = {
  let by = _by-name(data)
  let grouped = groups.map(g => g.at(1)).flatten()
  for name in by.keys() {
    let n = grouped.filter(x => x == name).len()
    if n == 0 { panic("cli.json command not grouped in cli.typ: " + name) }
    if n > 1 { panic("cli.json command grouped more than once in cli.typ: " + name) }
  }
  for name in grouped {
    if name not in by { panic("cli.typ group references unknown command: " + name) }
  }
  let valid = ()
  for a in data.globals { valid.push("@global." + a.id) }
  for c in data.commands { valid += _cmd-keys(c, (c.name,)) }
  for key in cli-notes.keys() {
    if key not in valid { panic("cli-notes.typ key not in cli.json: " + key) }
  }
}
