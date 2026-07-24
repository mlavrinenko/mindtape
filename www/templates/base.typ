// Show rules for content rendered inside the layout.
//
// The layout renders the page title as <h1>, so content headings start at <h2>.
// Each heading gets a slug id so the on-page anchors and cross-page #fragments
// resolve.

#import "../utils/tola.typ": to-string

#let slug(s) = {
  let t = lower(if type(s) == str { s } else { to-string(s) })
  let r = ""
  let prev-dash = true
  for c in t.clusters() {
    let ok = (c >= "a" and c <= "z") or (c >= "0" and c <= "9")
    if ok {
      r = r + c
      prev-dash = false
    } else if not prev-dash {
      r = r + "-"
      prev-dash = true
    }
  }
  if r.ends-with("-") { r = r.slice(0, r.len() - 1) }
  if r.starts-with("-") { r = r.slice(1) }
  r
}

#let base(body) = {
  // h1 is the page title (from the layout); content headings shift down one.
  show heading.where(level: 1): it => html.elem("h2", attrs: (
    id: slug(it.body),
  ))[#it.body]
  show heading.where(level: 2): it => html.elem("h3", attrs: (
    id: slug(it.body),
  ))[#it.body]
  show heading.where(level: 3): it => html.elem("h4", attrs: (
    id: slug(it.body),
  ))[#it.body]

  body
}
