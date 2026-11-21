// Rendering helpers shared across content pages.

#import "site.typ": (
  MARKDOWN, NAV, PKG, SITE, md-copy-button, mindtape, page-meta, render-summary,
  section-head, u,
)
#import "release.typ": cli-crate
#import "demo.typ": (
  fixture-root, listing-capture, showcase-file, showcase-intro, showcase-path,
)

// Inline package name, e.g. `@local/mindtape:0.3.0` — always version-synced.
#let pkg = raw(PKG)

// `html.elem(tag, attrs: attrs, body)`, or the bare body in the Markdown copy
// (mode.typ), where typlite drops a div or a span together with its content.
#let el(tag, attrs: (:), body) = if MARKDOWN { body } else {
  html.elem(tag, attrs: attrs, body)
}

// A dark terminal panel holding plain text, with leading and trailing blank
// lines trimmed. The head names the panel, so a capture reads as one surface
// with the code panels rather than a bare black box.
#let _terminal-panel(body) = {
  let text = raw(body.trim("\n", repeat: true), block: true)
  if MARKDOWN { text } else {
    html.elem("div", attrs: (class: "terminal"), {
      html.elem("div", attrs: (class: "terminal-head"), {
        html.elem("i")
        "TERMINAL"
      })
      text
    })
  }
}

// ── ANSI color ──────────────────────────────────────────────────────────────
// A capture keeps the SGR escapes `mt` printed in a terminal. terminal-ansi()
// turns each run into an HTML <span> with an inline color, the way Typst's
// HTML export renders syntax highlighting, so the text stays selectable and the
// browser draws the status glyphs. One-Dark-ish palette, readable on --term-bg.
// A reset, or no color, falls back to the panel's own --term-fg.
#let _ansi-fg = (
  "30": "#5c6370",
  "31": "#e06c75",
  "32": "#98c379",
  "33": "#e5c07b",
  "34": "#61afef",
  "35": "#c678dd",
  "36": "#56b6c2",
  "37": "#dcdfe4",
  "90": "#7f848e",
  "91": "#e06c75",
  "92": "#98c379",
  "93": "#e5c07b",
  "94": "#61afef",
  "95": "#c678dd",
  "96": "#56b6c2",
  "97": "#ffffff",
)
// One SGR parameter list ("1;31", "0", ...) applied to the current state, a
// (color, bold) pair.
#let _sgr-apply(state, codes) = {
  let (col, bold) = state
  for p in codes.split(";") {
    if p == "0" or p == "" {
      col = none
      bold = false
    } else if p == "1" {
      bold = true
    } else if p == "22" {
      bold = false
    } else if p == "39" {
      col = none
    } else if p in _ansi-fg {
      col = _ansi-fg.at(p)
    }
  }
  (col, bold)
}
#let _sgr-re = regex("\u{1b}\[([0-9;]*)m")
// ANSI text -> content: plain runs as text, colored runs as styled spans.
#let _ansi-spans(body) = {
  let out = ()
  let pos = 0
  let state = (none, false)
  let run(state, text) = {
    let (col, bold) = state
    let style = ()
    if col != none { style.push("color: " + col) }
    if bold { style.push("font-weight: bold") }
    if style == () { text } else {
      html.elem("span", attrs: (style: style.join("; ")), text)
    }
  }
  for m in body.matches(_sgr-re) {
    if m.start > pos { out.push(run(state, body.slice(pos, m.start))) }
    state = _sgr-apply(state, m.captures.at(0))
    pos = m.end
  }
  if pos < body.len() { out.push(run(state, body.slice(pos))) }
  out.join()
}
// The same ANSI-bearing string, stripped of its SGR escapes.
#let _ansi-strip(body) = body.replace(_sgr-re, "")

// ── Capture records ──────────────────────────────────────────────────────────
// www/capture.sh writes www/generated/<slug>.json: the project the commands ran
// in, and per command the line bash ran, what the terminal showed and the exit
// code. A page passes the slug and these helpers print each command from the
// record, so a page cannot show a command other than the one that ran.
#let _transcript(record) = {
  let lines = record.steps.map(step => {
    let out = step.out.trim("\n", repeat: true)
    if out == "" { "$ " + step.cmd } else { "$ " + step.cmd + "\n" + out }
  })
  lines.join("\n")
}

// Each command of a capture after a `$` prompt, then what it printed.
#let terminal(slug) = _terminal-panel(_ansi-strip(_transcript(json(
  "../generated/" + slug + ".json",
))))

// The exit code of the refusal a capture shows: the last command that exited
// non-zero. A session can end on a command that passes, such as the filing
// that follows a refusal, so the last command's code would report a success
// as a refusal. The build fails when every command passed.
//
// A capture that shows two refusals names one with `step`, the command's
// index from 0. The build fails when the capture has no such command, or when
// it exited 0.
#let exit-code(slug, step: none) = {
  let steps = json("../generated/" + slug + ".json").steps
  if step == none {
    let failed = steps.filter(s => s.code != 0)
    assert(
      failed.len() > 0,
      message: "capture " + slug + " has no command that exited non-zero",
    )
    return raw(str(failed.last().code))
  }
  let picked = steps.at(step, default: none)
  assert(
    picked != none,
    message: "capture " + slug + " has no command " + str(step),
  )
  assert(
    picked.code != 0,
    message: "capture " + slug + " command " + str(step) + " exited 0",
  )
  raw(str(picked.code))
}

// One line of what a capture printed, as inline code: the first line, color
// stripped, that matches the regex `pattern`. The build fails when no line
// matches, so a quoted diagnostic cannot outlive its text.
#let capture-line(slug, pattern) = {
  let record = json("../generated/" + slug + ".json")
  let lines = record
    .steps
    .map(step => _ansi-strip(step.out).split("\n"))
    .flatten()
  let hit = lines.find(line => line.contains(regex(pattern)))
  assert(
    hit != none,
    message: "capture " + slug + " has no line matching " + pattern,
  )
  raw(hit.trim())
}

// The commands of a capture alone, as a block a reader can copy.
#let commands(slug) = raw(
  json("../generated/" + slug + ".json").steps.map(step => step.cmd).join("\n"),
  lang: "sh",
  block: true,
)

// `terminal()` with the color the terminal showed: the same panel and the same
// `$ command` lines, with each colored run as a span in a selectable `pre`.
// The Markdown copy has no color, so it shows `terminal()`.
#let terminal-ansi(slug, head: none) = if MARKDOWN { terminal(slug) } else {
  html.elem("div", attrs: (class: "terminal"), {
    if head != none {
      html.elem("div", attrs: (class: "terminal-head"), {
        html.elem("i")
        head
      })
    }
    html.elem("pre", html.elem("code", _ansi-spans(_transcript(json(
      "../generated/" + slug + ".json",
    )))))
  })
}

// The hero's task, its `mt ls` row and its typeset card are read from the live
// fixture and its captures under generated/ (see www/generate.sh), so none
// drift from reality: the source is the verbatim task file, the terminal is
// real `mt ls` output, and the card is that same task compiled by the `task`
// show rule.
#let _hero-source = read("../generated/hero-source.typ")

// ── Departure board ─────────────────────────────────────────────────────────
// The board is the same `mt ls` a reader gets: the table capture supplies the
// rows, their readiness and their footer count, and the JSON run of the same
// default view supplies the blocker a row names. Nothing is typed in.
#let _take(list, n) = if list.len() > n { list.slice(0, n) } else { list }

#let _ls-lines = _ansi-strip(
  json("../generated/hero-ls.json").steps.at(0).out,
).split("\n")

#let _board-tasks = json("../generated/home-board-tasks.json").tasks

#let _board-rows = {
  let rows = ()
  for line in _ls-lines {
    let cells = line.split(regex("\\s{2,}")).map(s => s.trim())
    if cells.len() < 5 { continue }
    let ready = cells.at(2)
    if not (ready.contains("ready") or ready.contains("blocked")) { continue }
    let title = cells.last()
    let task = _board-tasks.find(t => t.title == title)
    let needs = if (
      task != none
        and task.why.len() > 0
        and task.why.at(0).kind == "depends-on"
    ) { task.why.at(0).stem } else { none }
    rows.push((
      slug: cells.at(0),
      status: cells.at(1),
      blocked: ready.contains("blocked"),
      wip: cells.at(1) == "wip",
      priority: cells.at(3).split(" ").last(),
      due: if cells.len() >= 6 { cells.at(4) } else { "" },
      title: title,
      needs: needs,
    ))
  }
  rows
}

// The three-letter slug a blocker is listed under, or the stem when the board
// does not list it.
#let _slug-for(id) = {
  let task = _board-tasks.find(t => t.id == id)
  if task == none { return id }
  let row = _board-rows.find(r => r.title == task.title)
  if row == none { return id }
  row.slug
}

#let _board-counts = {
  let line = _ls-lines.find(l => l.contains("default view"))
  assert(line != none, message: "hero-ls.json has no default-view footer")
  let hit = line.matches(regex("([0-9]+) of ([0-9]+) tasks")).last()
  assert(hit != none, message: "hero-ls.json footer has no task count")
  (visible: hit.captures.at(0), total: hit.captures.at(1))
}

#let _board() = {
  let counts = _board-counts
  let ready = _board-rows.filter(r => not r.blocked)
  let blocked = _board-rows.filter(r => r.blocked)
  html.elem("div", attrs: (class: "hero-board"), {
    html.elem("div", attrs: (class: "caption"), {
      html.elem("span", {
        html.elem("b", "DEPARTURES")
        " / backlog"
      })
      html.elem("span", upper(
        counts.visible + " OF " + counts.total + " TASKS",
      ))
    })
    html.elem("figure", attrs: (class: "board"), {
      html.elem("figcaption", attrs: (class: "board-head"), {
        html.elem("span", "MT LS / DEFAULT VIEW")
        html.elem("span", {
          html.elem("b", "●")
          " LIVE"
        })
      })
      html.elem("div", attrs: (class: "board-cols"), {
        html.elem("span", "Status")
        html.elem("span", "Task")
        html.elem("span", "Due")
      })
      html.elem("ol", attrs: (class: "board-rows"), {
        for row in _take(ready, 4) + _take(blocked, 2) {
          let cls = (
            "st"
              + if row.blocked { " is-blocked" } else if row.wip {
                " is-wip"
              } else { "" }
          )
          let label = if row.blocked { "blocked" } else if row.wip {
            "wip"
          } else { "ready" }
          let needs = if row.needs != none { _slug-for(row.needs) } else {
            none
          }
          let small = (
            row.slug
              + " · "
              + row.status
              + " · "
              + row.priority
              + if needs != none { " · needs " + needs } else { "" }
          )
          html.elem("li", {
            html.elem("span", attrs: (class: cls), {
              html.elem("i")
              label
            })
            html.elem("span", attrs: (class: "board-task"), {
              html.elem("b", row.title)
              html.elem("small", small)
            })
            html.elem("span", attrs: (class: "due"), if row.due == "" {
              "—"
            } else { row.due.slice(5) })
          })
        }
      })
      html.elem("div", attrs: (class: "board-foot"), {
        html.elem("span", "default view")
        html.elem("span", counts.visible + " of " + counts.total)
      })
    })
  })
}

// A numbered section head: the transit line's marker for a new stretch of the
// home page. The note at the right names where the section's material came
// from, so a reader can check it.

#let _hero-copy() = {
  let overview = page-meta("/")
  let label = if overview == none { "Overview" } else { upper(overview.label) }
  html.elem("div", attrs: (class: "hero-copy"), {
    html.elem("p", attrs: (class: "kicker"), {
      html.elem("span", attrs: (class: "num"), "01")
      " " + upper(NAV.at(0).title) + " / " + label
      html.elem("span", attrs: (class: "bar"))
    })
    html.elem("h1", mindtape)
    html.elem("p", attrs: (class: "lede"), SITE.tagline)
    html.elem("p", attrs: (class: "pitch"), render-summary(SITE.pitch))
    html.elem("div", attrs: (class: "hero-cta"), {
      html.elem(
        "a",
        attrs: (class: "btn", href: u("/quickstart/")),
        [Get started #html.elem("span", "→")],
      )
      html.elem(
        "a",
        attrs: (class: "btn btn-ghost", href: u("/concepts/")),
        [How it works],
      )
    })
    html.elem("p", attrs: (class: "install"), {
      "INSTALL "
      html.elem("code", "cargo install " + cli-crate)
    })
  })
}

// The site's own line into the listing: shorter than README's, which also
// points at the rest of the backlog.
#let _site-listing-intro = [And here's how `mt ls` lists it.]

// The task file, then how `mt ls` lists it. The typeset card is an inline SVG,
// which typlite cannot write, so only the HTML hero adds it.
#let _md-hero() = [
  #SITE.tagline

  #render-summary(SITE.pitch)

  #link(u("/quickstart/"))[Get started] · #link(u("/concepts/"))[How it works]

  #showcase-intro

  #raw(_hero-source, lang: "typ", block: true)

  #_site-listing-intro

  #terminal(listing-capture)

  The same file also typesets as a document.
]

#let _html-hero() = {
  html.elem("section", attrs: (class: "hero"), {
    _hero-copy()
    _board()
  })
  html.elem("section", attrs: (class: "proof"), {
    section-head(
      "02",
      [Here's a real task.],
      note: "SOURCE · " + upper(showcase-file),
    )
    html.elem("div", attrs: (class: "code"), {
      html.elem("div", attrs: (class: "code-head"), {
        html.elem("span", attrs: (class: "sq"))
        html.elem("span", "TASK FILE")
        html.elem("span", attrs: (class: "path"), showcase-path)
        html.elem(
          "button",
          attrs: (
            class: "copy",
            type: "button",
            "aria-label": "Copy task file",
          ),
          "COPY",
        )
      })
      raw(_hero-source, lang: "typ", block: true)
    })
    // The card stays folded: one line says the file typesets, and a click
    // shows it. Read the SVG bytes rather than `image("…path…")`: the card is
    // build-time content inlined as a data-uri, not a served asset, so a path
    // literal would trip tola's asset-link validator (generated/ is never
    // copied to public/).
    html.elem("details", attrs: (class: "proof-typeset"), {
      html.elem("summary", attrs: (class: "proof-foot"), {
        html.elem("span", "THE SAME FILE ALSO TYPESETS AS A DOCUMENT")
        html.elem("span", attrs: (class: "proof-open"), "Take a look →")
      })
      html.elem("div", attrs: (class: "typeset-card"), image(
        read("../generated/hero-card.svg", encoding: none),
        format: "svg",
        alt: "The task above, typeset as a document",
      ))
    })
  })
  html.elem("section", attrs: (class: "mt-list"), {
    section-head(
      "03",
      _site-listing-intro,
      note: "TERMINAL · " + fixture-root,
    )
    terminal-ansi(listing-capture, head: "TERMINAL · " + fixture-root)
  })
}

#let hero() = if MARKDOWN { _md-hero() } else { _html-hero() }

// Callout box. kind ∈ note | tip | warn. The Markdown copy shows a blockquote
// that opens with the kind.
#let _callout-label = (note: "Note", tip: "Tip", warn: "Warning")
#let callout(kind, body) = if MARKDOWN {
  quote(block: true)[#strong(_callout-label.at(kind) + ":") #body]
} else {
  html.elem("div", attrs: (class: "callout callout-" + kind), body)
}

// ── Diagrams ─────────────────────────────────────────────────────────────────
// render_dot in www/capture.sh draws a graph as www/generated/<slug>.svg with
// Graphviz. diagram() rebuilds that SVG as inline elements rather than an
// <img>, so the stylesheet can color it for the light and the dark theme.
// Typst parses a comment in the file as an element with an empty tag, which is
// dropped. So is the graph's own <title>, which a browser would show as a
// tooltip over the whole drawing.
#let _svg-elem(node) = if type(node) == str {
  if node.trim() != "" { node }
} else if node.tag != "" {
  let children = node.children
  if node.attrs.at("class", default: "") == "graph" {
    children = children.filter(c => type(c) == str or c.tag != "title")
  }
  html.elem(node.tag, attrs: node.attrs, children.map(_svg-elem).join())
}

// The text of a node's first <tag> child, or none.
#let _svg-child-text(node, tag) = {
  let hit = node.children.find(c => type(c) == dictionary and c.tag == tag)
  if hit != none { hit.children.filter(c => type(c) == str).join() }
}

// Every edge as text, "<from> <label> <to>", with "to" for an unlabeled
// edge. Graphviz titles an edge's group "<from>-><to>".
#let _svg-edges(node) = if type(node) != dictionary { () } else if (
  node.attrs.at("class", default: "") == "edge"
) {
  let (from, to) = _svg-child-text(node, "title").split("->")
  let label = _svg-child-text(node, "text")
  if label == none { label = "to" }
  (from + " " + label + " " + to,)
} else { node.children.map(_svg-edges).flatten() }

// The graph <slug> with a caption. Its aria-label lists every edge, so a
// screen reader, or a renderer that drops the SVG, still has the graph as
// text. The Markdown copy shows the caption and that edge list.
#let diagram(slug, caption) = {
  let root = xml("../generated/" + slug + ".svg").find(n => (
    type(n) == dictionary and n.tag == "svg"
  ))
  if MARKDOWN {
    return [#caption #parbreak() #list(.._svg-edges(root))]
  }
  let label = "Edges: " + _svg-edges(root).join("; ") + "."
  html.elem("figure", attrs: (class: "diagram"), {
    html.elem(
      "svg",
      attrs: root.attrs + (role: "img", aria-label: label),
      root.children.map(_svg-elem).join(),
    )
    html.elem("figcaption", caption)
  })
}

// ── Language server ──────────────────────────────────────────────────────────
// www/lsp-capture.py runs tinymist on examples/lsp and writes what it returned
// to www/generated/lsp.json. The block below draws that record the way any
// editor shows it, without naming one: the source with a wavy line under each
// problem, the menu under the cursor, and the list of problems.

// The source lines as a Typst listing. The HTML block joins several listings
// into one surface, so a listing with no lines draws nothing.
#let _lsp-listing(lines) = if lines.len() > 0 {
  raw(lines.join("\n"), lang: "typ", block: true)
}

// A problem as an editor lists it: where, then what.
#let _lsp-where(problem) = (
  problem.file + ":" + str(problem.line) + ":" + str(problem.column)
)

// Source lines `from` to `to` (0-based, end excluded), with a wavy line under
// each problem's range on them, the way an editor marks it. A range that runs
// past its first line is marked to that line's end. The marked text repeats
// the source in a transparent color, so the line sits under the same
// characters in the listing's monospace font.
#let _lsp-marked(lines, problems, from, to) = {
  let shown = from
  let here = problems.filter(p => p.line > from and p.line <= to)
  for p in here.sorted(key: p => p.line) {
    _lsp-listing(lines.slice(shown, p.line))
    shown = p.line
    let chars = lines.at(p.line - 1).codepoints()
    let end = if p.end_line == p.line { p.end_column - 1 } else { chars.len() }
    html.elem("div", attrs: (class: "lsp-squiggle"), html.elem(
      "span",
      attrs: (
        class: "lsp-" + p.severity,
        style: "--lsp-column: " + str(p.column - 1),
      ),
      chars.slice(p.column - 1, end).join(),
    ))
  }
  _lsp-listing(lines.slice(shown, to))
}

// The cursor on its blank line and the fields tinymist offered there, as a
// menu under the cursor.
#let _lsp-menu(record) = {
  let column = "--lsp-column: " + str(record.column - 1)
  html.elem("div", attrs: (class: "lsp-cursor", style: column), html.elem(
    "span",
    attrs: (class: "lsp-caret"),
  ))
  html.elem(
    "ul",
    attrs: (class: "lsp-menu", style: column),
    record
      .items
      .map(name => html.elem("li", {
        html.elem("span", attrs: (class: "lsp-kind"), "field")
        html.elem("span", name)
      }))
      .join(),
  )
}

// The task file as tinymist saw it: the problems it published while it
// compiled the file, and the fields it offered at the cursor.
#let lsp-demo() = {
  let record = json("../generated/lsp.json")
  let lines = record.source.trim("\n", at: end, repeat: true).split("\n")
  let caption = [#record.server · #raw(record.file)]
  let problems = record.diagnostics.map(p => [
    #p.severity at #raw(_lsp-where(p)): #p.message
  ])
  if MARKDOWN {
    return [
      #caption

      #_lsp-listing(lines)

      Fields offered at line #record.line: #record.items.map(raw).join[, ].

      #list(..problems)
    ]
  }
  let cursor = record.line - 1
  html.elem("figure", attrs: (class: "lsp"), {
    html.elem("figcaption", caption)
    html.elem("div", attrs: (class: "lsp-code"), {
      _lsp-marked(lines, record.diagnostics, 0, cursor)
      _lsp-menu(record)
      _lsp-marked(lines, record.diagnostics, cursor + 1, lines.len())
    })
    html.elem(
      "ul",
      attrs: (class: "lsp-problems"),
      record
        .diagnostics
        .map(p => html.elem("li", attrs: (class: "lsp-" + p.severity), {
          html.elem("span", attrs: (class: "lsp-severity"), p.severity)
          html.elem("span", attrs: (class: "lsp-message"), p.message)
          html.elem("span", attrs: (class: "lsp-where"), _lsp-where(p))
        }))
        .join(),
    )
  })
}
