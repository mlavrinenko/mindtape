// Rendering helpers shared across content pages.

#import "site.typ": MARKDOWN, PKG, u
#import "demo.typ": intro, tagline

// Inline package name, e.g. `@local/mindtape:0.2.0` — always version-synced.
#let pkg = raw(PKG)

// `html.elem(tag, attrs: attrs, body)`, or the bare body in the Markdown copy
// (mode.typ), where typlite drops a div or a span together with its content.
#let el(tag, attrs: (:), body) = if MARKDOWN { body } else {
  html.elem(tag, attrs: attrs, body)
}

// A dark terminal panel holding plain text, with leading and trailing blank
// lines trimmed.
#let _terminal-panel(body) = el(
  "div",
  attrs: (class: "terminal"),
  raw(body.trim("\n", repeat: true), block: true),
)

// ── ANSI colour ──────────────────────────────────────────────────────────────
// A capture keeps the SGR escapes `mt` printed in a terminal. terminal-ansi()
// turns each run into an HTML <span> with an inline colour, the way Typst's
// HTML export renders syntax highlighting, so the text stays selectable and the
// browser draws the status glyphs. One-Dark-ish palette, readable on --term-bg.
// A reset, or no colour, falls back to the panel's own --term-fg.
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
// (colour, bold) pair.
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
// ANSI text -> content: plain runs as text, coloured runs as styled spans.
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

// The exit code of a capture's last command, for a page that shows a refusal.
#let exit-code(slug) = raw(str(
  json("../generated/" + slug + ".json").steps.last().code,
))

// One line of what a capture printed, as inline code: the first line, colour
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

// `terminal()` with the colour the terminal showed: the same panel and the same
// `$ command` lines, with each coloured run as a span in a selectable `pre`.
// The Markdown copy has no colour, so it shows `terminal()`.
#let terminal-ansi(slug) = if MARKDOWN { terminal(slug) } else {
  html.elem(
    "div",
    attrs: (class: "terminal"),
    html.elem("pre", html.elem("code", _ansi-spans(_transcript(json(
      "../generated/" + slug + ".json",
    ))))),
  )
}

// All three hero faces are read from the live fixture and its captures under
// generated/ (see www/generate.sh) — so none drift from reality: source is the
// verbatim task file, terminal is real `mt ls` output, and the typeset card is
// that same task genuinely compiled by the `task` show rule.
#let _hero-source = read("../generated/hero-source.typ")

// One face of the hero. The Markdown copy shows its caption as a paragraph.
#let face(kind, caption, body) = if MARKDOWN [#caption #parbreak() #body] else {
  html.elem(
    "figure",
    attrs: (class: "face face-" + kind),
    {
      html.elem("figcaption", [#caption])
      html.elem("div", attrs: (class: "face-body"), body)
    },
  )
}

// The faces both copies show. The typeset card is an inline SVG, which
// typlite cannot write, so only the HTML hero adds it.
#let _text-faces() = {
  face("terminal", "terminal", terminal-ansi("hero-ls"))
  face("source", "source · throttle-failed-logins-per-ip.typ", raw(
    _hero-source,
    lang: "typ",
    block: true,
  ))
}

// typlite joins two `par` calls into one line, so the paragraphs are markup.
#let _md-hero() = [
  #tagline

  #intro

  #link(u("/quickstart/"))[Get started] · #link(u("/concepts/"))[How it works]

  #_text-faces()
]

#let _html-hero() = html.elem("section", attrs: (class: "hero"), {
  html.elem("div", attrs: (class: "hero-lede"), {
    html.elem("h1", tagline)
    html.elem("p", attrs: (class: "hero-sub"), intro)
    html.elem("div", attrs: (class: "hero-cta"), {
      html.elem(
        "a",
        attrs: (class: "btn", href: u("/quickstart/")),
        [Get started],
      )
      html.elem(
        "a",
        attrs: (class: "btn ghost", href: u("/concepts/")),
        [How it works],
      )
    })
  })
  html.elem("div", attrs: (class: "faces"), {
    _text-faces()
    // Read the SVG bytes rather than `image("…path…")`: the card is build-time
    // content inlined as a data-uri, not a served asset, so a path literal would
    // trip tola's asset-link validator (generated/ is never copied to public/).
    face("card", "typeset · rendered", image(
      read("../generated/hero-card.svg", encoding: none),
      format: "svg",
    ))
  })
  html.elem("p", attrs: (class: "faces-note"))[
    One file, three faces: source you edit, a row you query, a document you can typeset.
  ]
})

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
// <img>, so the stylesheet can colour it for the light and the dark theme.
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

// Every edge as text, "<from> <label> <to>", with "to" for an unlabelled
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
// to www/generated/lsp-completion.json and lsp-diagnostics.json. The blocks
// below draw a record the way any editor shows it, without naming one: the
// source, the menu under the cursor, and the list of problems.

#let _lsp-record(slug) = json("../generated/" + slug + ".json")

// The source lines as a Typst listing. The HTML block joins several listings
// into one surface, so a listing with no lines draws nothing.
#let _lsp-listing(lines) = if lines.len() > 0 {
  raw(lines.join("\n"), lang: "typ", block: true)
}

// The task file with the cursor on its blank line, and the fields tinymist
// offered there, as a menu under the cursor.
#let lsp-completion() = {
  let record = _lsp-record("lsp-completion")
  let lines = record.source.trim("\n", at: end, repeat: true).split("\n")
  let caption = [#record.server · completion · #raw(record.file), line #record.line]
  if MARKDOWN {
    return [
      #caption

      #_lsp-listing(lines)

      #list(..record.items.map(raw))
    ]
  }
  let column = "--lsp-column: " + str(record.column - 1)
  html.elem("figure", attrs: (class: "lsp"), {
    html.elem("figcaption", caption)
    html.elem("div", attrs: (class: "lsp-code"), {
      _lsp-listing(lines.slice(0, record.line - 1))
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
      _lsp-listing(lines.slice(record.line))
    })
  })
}

// A problem as an editor lists it: where, then what.
#let _lsp-where(problem) = (
  problem.file + ":" + str(problem.line) + ":" + str(problem.column)
)

// The source with a wavy line under each problem's range, the way an editor
// marks it. A range that runs past its first line is marked to that line's end.
// The marked text repeats the source in a transparent colour, so the line sits
// under the same characters in the listing's monospace font.
#let _lsp-marked(lines, problems) = {
  let shown = 0
  for p in problems.sorted(key: p => p.line) {
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
  _lsp-listing(lines.slice(shown))
}

// The misspelled task file, marked where tinymist placed each diagnostic it
// published while it compiled that file, and the list of those problems.
#let lsp-diagnostics() = {
  let record = _lsp-record("lsp-diagnostics")
  let lines = record.source.trim("\n", at: end, repeat: true).split("\n")
  let caption = [#record.server · diagnostics · #raw(record.file)]
  if MARKDOWN {
    return [
      #caption

      #_lsp-listing(lines)

      #list(..record.diagnostics.map(p => [
        #p.severity at #raw(_lsp-where(p)): #p.message
      ]))
    ]
  }
  html.elem("figure", attrs: (class: "lsp"), {
    html.elem("figcaption", caption)
    html.elem("div", attrs: (class: "lsp-code"), _lsp-marked(
      lines,
      record.diagnostics,
    ))
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
