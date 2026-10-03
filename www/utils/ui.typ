// Rendering helpers shared across content pages.

#import "site.typ": PKG, u

// Inline package name, e.g. `@local/mindtape:0.2.0` — always version-synced.
#let pkg = raw(PKG)

// A dark terminal panel holding plain text, with leading and trailing blank
// lines trimmed.
#let _terminal-panel(body) = html.elem(
  "div",
  attrs: (class: "terminal"),
  raw(body.trim("\n", repeat: true), block: true),
)

// ── Colourised terminal (ANSI → inline SVG) ──────────────────────────────────
// tola / Typst HTML export exposes no raw-HTML escape hatch and drops
// `text(fill:)` colour, so the only way to render `mt`'s ANSI-coloured output in
// colour is `html.frame` (native content → inline <svg>). The generator captures
// the colour-bearing transcripts with `--color always` (www/generate.sh); this
// parses the SGR runs and lays them out as a monospace block on the dark panel.
// One-Dark-ish palette, readable on --term-bg; an SGR reset falls to --term-fg.
#let _term-fg = rgb("#d6dee6")
#let _term-bg = rgb("#0f151b")
#let _ansi-fg = (
  "30": rgb("#5c6370"),
  "31": rgb("#e06c75"),
  "32": rgb("#98c379"),
  "33": rgb("#e5c07b"),
  "34": rgb("#61afef"),
  "35": rgb("#c678dd"),
  "36": rgb("#56b6c2"),
  "37": rgb("#dcdfe4"),
  "90": rgb("#7f848e"),
  "91": rgb("#e06c75"),
  "92": rgb("#98c379"),
  "93": rgb("#e5c07b"),
  "94": rgb("#61afef"),
  "95": rgb("#c678dd"),
  "96": rgb("#56b6c2"),
  "97": rgb("#ffffff"),
)
// One SGR parameter list ("1;31", "0", …) → the current foreground colour.
#let _sgr-color(codes) = {
  let col = _term-fg
  for p in codes.split(";") {
    if p == "0" or p == "" { col = _term-fg } else if p in _ansi-fg {
      col = _ansi-fg.at(p)
    }
  }
  col
}
// The SGR escape this module understands, shared by the colour splitter below
// and `_ansi-strip`, so the "what counts as an SGR run" definition lives once.
#let _sgr-re = regex("\u{1b}\[([0-9;]*)m")
// ANSI text → a monospace block of coloured runs (whitespace significant,
// newlines → linebreaks), split on the SGR escape via regex match offsets.
#let _ansi-block(body) = {
  let runs = ()
  let pos = 0
  let cur = _term-fg
  for m in body.matches(_sgr-re) {
    if m.start > pos { runs.push((cur, body.slice(pos, m.start))) }
    cur = _sgr-color(m.captures.at(0))
    pos = m.end
  }
  if pos < body.len() { runs.push((cur, body.slice(pos))) }
  let lines = ()
  let line = ()
  for (col, seg) in runs {
    let parts = seg.split("\n")
    for (i, part) in parts.enumerate() {
      if i > 0 {
        lines.push(line)
        line = ()
      }
      if part != "" { line.push((col, part)) }
    }
  }
  lines.push(line)
  set text(font: "DejaVu Sans Mono", size: 8pt, fill: _term-fg)
  set par(leading: 0.65em)
  lines
    .map(l => l.map(((c, t)) => text(fill: c, raw(t))).join())
    .join(linebreak())
}
// The same ANSI-bearing string, stripped of its SGR escapes — the accessible
// text twin. Sourced from the very string every call site already passes (the
// `--color always` `.ansi` capture), so it can never diverge from the image:
// no caller change, no second `.txt` argument to keep in sync.
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

// The colour terminal: the same dark panel as `terminal()`, but the visible
// body is the html.frame'd SVG — an image, not selectable text (the copyable
// commands live in the sibling `sh` code blocks). Paired with a visually
// hidden, real-text twin (`.sr-only`, components.css) so assistive tech gets
// the actual transcript instead of nothing; the SVG itself is `aria-hidden`
// so the transcript isn't announced twice.
#let terminal-ansi(body) = {
  let trimmed = body.trim("\n", repeat: true)
  html.elem(
    "div",
    attrs: (class: "terminal terminal-svg"),
    {
      html.elem("div", attrs: (class: "sr-only"), raw(
        _ansi-strip(trimmed),
        block: true,
      ))
      html.elem(
        "div",
        attrs: ("aria-hidden": "true"),
        html.frame(block(
          fill: _term-bg,
          inset: (x: 14pt, y: 12pt),
          _ansi-block(trimmed),
        )),
      )
    },
  )
}

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
    html.elem("h1")[Tasks as Typst files.]
    html.elem("p", attrs: (class: "hero-sub"))[
      MindTape is a daemon-free, file-based task tracker. One task per #raw(".typ")
      file, live-evaluated on every command — no daemon, no database, no lock-in.
    ]
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
    face("terminal", "terminal · mt ls", terminal-ansi(
      read("../generated/ls.ansi"),
    ))
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
