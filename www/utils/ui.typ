// Rendering helpers shared across content pages.

#import "site.typ": PKG, u

// Inline package name, e.g. `@local/mindtape:0.2.0` — always version-synced.
#let pkg = raw(PKG)

// A `.typ` code block whose first line is the version-synced import, followed
// by `rest`. Use once per page for a full example; snippets elsewhere omit the
// import to stay DRY.
#let mt-import(rest) = raw("#import \"" + PKG + "\": *\n\n" + rest, lang: "typ", block: true)

// A dark terminal panel holding a plain (un-highlighted) transcript. Leading /
// trailing blank lines are trimmed.
#let terminal(body) = html.elem(
  "div",
  attrs: (class: "terminal"),
  raw(body.trim("\n", repeat: true), block: true),
)

#let pill(label, class: "") = html.elem("span", attrs: (class: ("pill " + class).trim()), [#label])
#let tag(label) = html.elem("span", attrs: (class: "tag"), [#label])

// The "typeset" face: a task rendered as a printed index card.
#let task-card = html.elem("article", attrs: (class: "task-card"), {
  html.elem("div", attrs: (class: "tc-head"), {
    pill("wip", class: "status-wip")
    pill("high", class: "band-high")
  })
  html.elem("h3", attrs: (class: "tc-title"), [Wire OAuth login])
  html.elem("p", attrs: (class: "tc-summary"), [Add the login flow behind the feature flag.])
  html.elem("dl", attrs: (class: "tc-meta"), {
    html.elem("div", html.elem("dt", [due]) + html.elem("dd", [2026-07-20]))
    html.elem("div", html.elem("dt", [estimate]) + html.elem("dd", [1 day]))
  })
  html.elem("div", attrs: (class: "tc-tags"), { tag("backend"); tag("auth") })
})

#let _hero-source = "#import \"" + PKG + "\": *

#show: task.with(
  title: \"Wire OAuth login\",
  status: wip(2026, 7, 6),
  priority: 8,
  estimate: (days: 1),
  tags: (\"backend\", \"auth\"),
  due: datetime(year: 2026, month: 7, day: 20),
)

== Summary
Add the login flow behind the feature flag."

#let _hero-terminal = "$ mt ls
#   STATUS    READY    PRIORITY  DUE         TITLE
1e  wip       ready    high      2026-07-20  Wire OAuth login
2g  proposed  blocked  normal                Write onboarding docs"

#let face(kind, caption, body) = html.elem("figure", attrs: (class: "face face-" + kind), {
  html.elem("figcaption", [#caption])
  html.elem("div", attrs: (class: "face-body"), body)
})

#let hero() = html.elem("section", attrs: (class: "hero"), {
  html.elem("div", attrs: (class: "hero-lede"), {
    html.elem("h1")[Tasks as Typst files.]
    html.elem("p", attrs: (class: "hero-sub"))[
      MindTape is a daemon-free, file-based task tracker. One task per #raw(".typ")
      file, live-evaluated on every command — no daemon, no database, no lock-in.
    ]
    html.elem("div", attrs: (class: "hero-cta"), {
      html.elem("a", attrs: (class: "btn", href: u("/quickstart/")), [Get started])
      html.elem("a", attrs: (class: "btn ghost", href: u("/concepts/")), [How it works])
    })
  })
  html.elem("div", attrs: (class: "faces"), {
    face("source", "source · wire-oauth.typ", raw(_hero-source, lang: "typ", block: true))
    face("terminal", "terminal · mt ls", terminal(_hero-terminal))
    face("card", "typeset · rendered", task-card)
  })
  html.elem("p", attrs: (class: "faces-note"))[
    One file, three faces: source you edit, a row you query, a document you can typeset.
  ]
})

// Callout box. kind ∈ note | tip | warn.
#let callout(kind, body) = html.elem("div", attrs: (class: "callout callout-" + kind), body)
