// Page chrome: <head>, header, docs sidebar, main, footer.

#import "../utils/site.typ": SITE, NAV, REPO, u

// Set data-theme before first paint (no flash), then a global toggle.
#let _theme-script = "(function(){var d=document.documentElement;var t=localStorage.getItem('theme');if(t){d.setAttribute('data-theme',t)}else if(window.matchMedia('(prefers-color-scheme:dark)').matches){d.setAttribute('data-theme','dark')}}());function toggleTheme(){var d=document.documentElement;var n=d.getAttribute('data-theme')==='dark'?'light':'dark';d.setAttribute('data-theme',n);localStorage.setItem('theme',n)}"

#let make-head(m) = {
  html.elem("meta", attrs: (charset: "utf-8"))
  html.elem("meta", attrs: (name: "viewport", content: "width=device-width, initial-scale=1"))
  let t = m.at("title", default: SITE.title)
  html.elem("title")[#(if t == SITE.title { t } else { t + " — " + SITE.title })]
  let desc = m.at("summary", default: SITE.tagline)
  if desc != none { html.elem("meta", attrs: (name: "description", content: desc)) }
  html.elem("link", attrs: (rel: "icon", href: "/assets/images/logo.svg", type: "image/svg+xml"))
  html.elem("link", attrs: (rel: "preconnect", href: "https://fonts.googleapis.com"))
  html.elem("link", attrs: (rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: ""))
  html.elem("link", attrs: (
    rel: "stylesheet",
    href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&family=Source+Serif+4:opsz,wght@8..60,400;8..60,600;8..60,700&display=swap",
  ))
  // Assets stay source-relative; tola applies the site prefix on build.
  html.elem("link", attrs: (rel: "stylesheet", href: "/assets/styles/main.css"))
  html.elem("link", attrs: (rel: "stylesheet", href: "/assets/styles/components.css"))
  html.script(_theme-script)
}

#let _brand = html.elem(
  "a",
  attrs: (href: u("/"), class: "brand"),
  // Logo as an accent-tinted inline mark (see .brand-mark in main.css); a real
  // block-level <img> would force a <p> inside the <a> and split the anchor.
  html.elem("span", attrs: (class: "brand-mark", role: "img", "aria-label": "MindTape"))
    + html.elem("span", attrs: (class: "brand-name"), [MindTape]),
)

#let _toggle = html.elem(
  "button",
  attrs: (class: "theme-toggle", type: "button", "aria-label": "Toggle theme", onclick: "toggleTheme()"),
  html.elem("span", attrs: (class: "theme-icon theme-icon-sun"), [☀])
    + html.elem("span", attrs: (class: "theme-icon theme-icon-moon"), [☾]),
)

#let _sidebar(active) = html.elem("aside", attrs: (class: "sidebar"), {
  html.elem("nav", attrs: ("aria-label": "Documentation"), {
    for group in NAV {
      html.elem("div", attrs: (class: "nav-group"), {
        html.elem("p", attrs: (class: "nav-title"), [#group.title])
        html.elem("ul", {
          for item in group.items {
            let attrs = (href: u(item.href))
            if active == item.href { attrs.insert("aria-current", "page") }
            html.elem("li", html.elem("a", attrs: attrs, [#item.label]))
          }
        })
      })
    }
  })
})

#let layout(body, meta: (:)) = {
  let title = meta.at("title", default: none)
  let active = meta.at("active", default: none)
  let home = meta.at("home", default: false)

  html.elem("header", attrs: (class: "site-header"), {
    _brand
    html.elem("div", attrs: (class: "header-right"), {
      html.elem("a", attrs: (href: REPO, class: "header-link"), [GitHub])
      _toggle
    })
  })

  html.elem("div", attrs: (class: "shell"), {
    _sidebar(active)
    html.elem("main", attrs: (class: if home { "home" } else { "" }), {
      if title != none and not home { html.elem("h1")[#title] }
      body
    })
  })

  html.elem("footer", attrs: (class: "site-footer"), {
    html.elem("p")[
      #SITE.title — #SITE.tagline · #link(REPO)[GitHub] · MIT
    ]
  })
}
