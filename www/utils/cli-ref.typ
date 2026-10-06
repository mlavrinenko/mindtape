// Render www/generated/cli.json, the command tree `mt _cli --json` reflects
// from clap, as the CLI pages: the index at /cli/ and one page per top-level
// command at /cli/<command>/. The JSON holds the facts. cli-notes.typ adds
// prose and captures by dotted key (`ls`, `ls.filter`, `config.show`,
// `config.show.origin`), and this file decides how both look.
//
// A command page is www/content/cli/<command>.typ:
//
//   #import "../../utils/cli-ref.typ": cli-command, cli-page
//   #show: cli-page.with("mv")
//   #cli-command("mv")
//
// Prose of the page's own goes before or after the `cli-command` call.
// `cli-page` fails the build unless the page renders its own command once,
// and `just check-doc-coverage` fails unless every command has a page.

#import "ui.typ": commands, terminal
#import "cli-notes.typ": cli-notes
#import "site.typ": u
#import "page.typ": page

#let CLI = json("../generated/cli.json")

#let _dim(body) = text(fill: rgb("#5b6570"), size: 0.9em, body)

// ── argument signature, e.g. `-n, --limit <LIMIT>` or `[FILTER]` ──────────────
#let _sig(arg) = {
  if arg.kind == "positional" {
    let name = if arg.value_name != none { arg.value_name } else {
      upper(arg.id)
    }
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
    if extra != "" {
      parbreak()
      _dim[#extra]
    }
  }
}

// ── one argument's description cell ──────────────────────────────────────────
#let _arg-desc(arg, key) = {
  _desc(arg.about, arg.long_about)
  if arg.possible_values != none {
    parbreak()
    _pvals(arg)
  }
  let meta = ()
  if arg.default != none { meta.push("default: " + arg.default) }
  if arg.multiple { meta.push("repeatable") }
  if meta.len() > 0 {
    parbreak()
    _dim(raw(meta.join("   ·   ")))
  }
  let note = cli-notes.at(key, default: none)
  if note != none {
    if "prose" in note {
      parbreak()
      note.prose
    }
    if "examples" in note { commands(note.examples) }
  }
}

#let _args-table(args, key-prefix) = {
  if args.len() == 0 { return }
  table(
    columns: (auto, 1fr),
    align: (left + top, left + top),
    stroke: none,
    inset: (x: 6pt, y: 5pt),
    ..args
      .map(arg => (raw(_sig(arg)), _arg-desc(arg, key-prefix + "." + arg.id)))
      .flatten()
  )
}

// ── capture records attached to a command via its note ──────────────────────
#let _outputs(note) = {
  if note == none { return }
  for slug in note.at("outputs", default: ()) { terminal(slug) }
}

// ── one command block (recursive over subcommands) ───────────────────────────
// The page title names the top-level command, so only a subcommand gets a
// heading: `mt config show` is a level-1 heading on the `config` page.
#let _command(cmd, path) = {
  let key = path.join(".")
  let note = cli-notes.at(key, default: none)
  if path.len() > 1 {
    heading(level: path.len() - 1, raw("mt " + path.join(" ")))
  }
  _desc(cmd.about, cmd.long_about)
  if note != none and "prose" in note {
    parbreak()
    note.prose
  }
  _args-table(cmd.args, key)
  if note != none and "examples" in note { commands(note.examples) }
  _outputs(note)
  for sub in cmd.subcommands { _command(sub, path + (sub.name,)) }
}

#let _by-name(data) = {
  let m = (:)
  for c in data.commands { m.insert(c.name, c) }
  m
}

#let _lookup(name) = {
  let by = _by-name(CLI)
  if name not in by {
    panic(
      "cli-ref: no command "
        + repr(name)
        + " in cli.json; known: "
        + by.keys().join(", "),
    )
  }
  by.at(name)
}

#let cli-route(name) = "/cli/" + name + "/"

// ── public: one command's page ───────────────────────────────────────────────
// The show rule a command page applies. It declares the page's route, label
// and title from the command name, and fails the build unless the page calls
// `cli-command` for that command exactly once.
#let cli-page(name, body) = {
  let _ = _lookup(name)
  let meta = (
    route: cli-route(name),
    label: "mt " + name,
    title: "mt " + name,
  )
  [#metadata(meta) <page-meta>]
  page(..meta, {
    body
    parbreak()
    [Every command also takes the #link(u("/cli/#global-flags"))[global flags].]
    context {
      let rendered = query(<cli-command>).map(m => m.value)
      if rendered != (name,) {
        panic(
          "cli page "
            + name
            + " must call cli-command("
            + repr(name)
            + ") once; it renders "
            + repr(rendered),
        )
      }
    }
  })
}

// The command's reference: its description, notes, arguments, examples,
// transcripts and subcommands.
#let cli-command(name) = {
  [#metadata(name) <cli-command>]
  _command(_lookup(name), (name,))
}

// ── public: the index, the global flags and the coverage gate ────────────────
#let _group-names(groups, title) = {
  for g in groups {
    if g.at(0) == title { return g.at(1) }
  }
  panic("cli-ref: unknown group " + title)
}

// The global flags, shown once on the index. clap's own `--help` repeats them
// on every command.
#let render-globals(data) = _args-table(data.globals, "@global")

// One group of the index: each command linked to its page, with its summary.
#let cli-group(data, groups, title) = {
  let by = _by-name(data)
  table(
    columns: (auto, 1fr),
    align: (left + top, left + top),
    stroke: none,
    inset: (x: 6pt, y: 5pt),
    .._group-names(groups, title)
      .map(name => (
        link(u(cli-route(name)), raw("mt " + name)),
        [#by.at(name).about],
      ))
      .flatten()
  )
}

// Every dotted key a note may legally attach to (commands + their args, recursive).
#let _cmd-keys(cmd, path) = {
  let key = path.join(".")
  let keys = (key,)
  for a in cmd.args { keys.push(key + "." + a.id) }
  for sub in cmd.subcommands { keys += _cmd-keys(sub, path + (sub.name,)) }
  keys
}

// Two-way completeness gate for the index: every command sits in exactly one
// group, and every note names a command or argument. The index links each
// grouped command to its page, so `tola validate` also fails on a command
// with no page.
#let cli-gate(data, groups) = {
  let by = _by-name(data)
  let grouped = groups.map(g => g.at(1)).flatten()
  for name in by.keys() {
    let n = grouped.filter(x => x == name).len()
    if n == 0 { panic("cli.json command not grouped in cli.typ: " + name) }
    if n > 1 {
      panic("cli.json command grouped more than once in cli.typ: " + name)
    }
  }
  for name in grouped {
    if name not in by {
      panic("cli.typ group references unknown command: " + name)
    }
  }
  let valid = ()
  for a in data.globals { valid.push("@global." + a.id) }
  for c in data.commands { valid += _cmd-keys(c, (c.name,)) }
  for key in cli-notes.keys() {
    if key not in valid { panic("cli-notes.typ key not in cli.json: " + key) }
  }
}
