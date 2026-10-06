// Rendering helpers shared across content pages.

#import "site.typ": PKG, u
#import "demo.typ": intro, tagline

// Inline package name, e.g. `@local/mindtape:0.2.0` — always version-synced.
#let pkg = raw(PKG)

// A dark terminal panel holding plain text, with leading and trailing blank
// lines trimmed.
#let _terminal-panel(body) = html.elem(
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

// The commands of a capture alone, as a block a reader can copy.
#let commands(slug) = raw(
  json("../generated/" + slug + ".json").steps.map(step => step.cmd).join("\n"),
  lang: "sh",
  block: true,
)

// `terminal()` with the colour the terminal showed: the same panel and the same
// `$ command` lines, with each coloured run as a span in a selectable `pre`.
#let terminal-ansi(slug) = html.elem(
  "div",
  attrs: (class: "terminal"),
  html.elem("pre", html.elem("code", _ansi-spans(_transcript(json(
    "../generated/" + slug + ".json",
  ))))),
)

// All three hero faces are read from the live fixture and its captures under
// generated/ (see www/generate.sh) — so none drift from reality: source is the
// verbatim task file, terminal is real `mt ls` output, and the typeset card is
// that same task genuinely compiled by the `task` show rule.
#let _hero-source = read("../generated/hero-source.typ")

#let face(kind, caption, body) = html.elem(
  "figure",
  attrs: (class: "face face-" + kind),
  {
    html.elem("figcaption", [#caption])
    html.elem("div", attrs: (class: "face-body"), body)
  },
)

#let hero() = html.elem("section", attrs: (class: "hero"), {
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
    face("terminal", "terminal", terminal-ansi("hero-ls"))
    face("source", "source · throttle-failed-logins-per-ip.typ", raw(
      _hero-source,
      lang: "typ",
      block: true,
    ))
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

// Callout box. kind ∈ note | tip | warn.
#let callout(kind, body) = html.elem(
  "div",
  attrs: (class: "callout callout-" + kind),
  body,
)

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
// text.
#let diagram(slug, caption) = {
  let root = xml("../generated/" + slug + ".svg").find(n => (
    type(n) == dictionary and n.tag == "svg"
  ))
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
