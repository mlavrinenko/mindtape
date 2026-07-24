// Page chrome: <head>, header, docs sidebar, main, footer.

#import "../utils/site.typ": SITE, NAV, REPO, u

// Set data-theme before first paint (no flash), then a global toggle.
#let _theme-script = "(function(){var d=document.documentElement;var t=localStorage.getItem('theme');if(t){d.setAttribute('data-theme',t)}else if(window.matchMedia('(prefers-color-scheme:dark)').matches){d.setAttribute('data-theme','dark')}}());function toggleTheme(){var d=document.documentElement;var n=d.getAttribute('data-theme')==='dark'?'light':'dark';d.setAttribute('data-theme',n);localStorage.setItem('theme',n)}"

// Bootstrap Pagefind's search UI. The bundle (UI JS/CSS + index) is generated
// post-build by `pagefind --site public/mindtape` (Justfile docs-build) into
// /mindtape/pagefind/, which is invisible to `tola build`/`validate`. Injecting
// the stylesheet + script at runtime (rather than static <link>/<script>) keeps
// tola's asset validator — which only knows build-time assets — from failing on
// them. Once the script loads, mount PagefindUI into the header #search box
// (waiting for the DOM if the async script wins the race), then make the
// results drawer behave like a popup: dismiss on outside-click / Escape via a
// `search--closed` class the CSS honours, and re-open when the input regains
// focus. Search is a production-build feature; `tola serve` has no index.
#let _search-init = "(function(){var p='" + u("/pagefind/") + "';" + "var l=document.createElement('link');l.rel='stylesheet';l.href=p+'pagefind-ui.css';document.head.appendChild(l);" + "var s=document.createElement('script');s.src=p+'pagefind-ui.js';s.onload=function(){var init=function(){" + "new PagefindUI({element:'#search',showSubResults:true,showImages:false,bundlePath:p});" + "var box=document.getElementById('search');if(!box)return;" + "document.addEventListener('click',function(e){if(!box.contains(e.target)){box.classList.add('search--closed');}});" + "box.addEventListener('focusin',function(){box.classList.remove('search--closed');});" + "document.addEventListener('keydown',function(e){if(e.key==='Escape'){box.classList.add('search--closed');box.blur&&box.blur();}});" + "};if(document.readyState!=='loading'){init();}else{document.addEventListener('DOMContentLoaded',init);}};document.head.appendChild(s);}());"

// Floating <thead> clone for wide tables like the comparison matrix. Plain
// CSS `position: sticky` on `thead th` can't work there: the `.matrix`
// wrapper is a horizontal scroll container (`overflow-x: auto`), and that
// clips block-axis sticky positioning for its descendants. No-op on pages
// without a `.matrix table thead` (guarded up front). This script tag sits
// in <head> (before <body> is parsed), so the setup runs on DOMContentLoaded
// rather than immediately.
#let _matrix-float-head-script = "document.addEventListener('DOMContentLoaded',function(){
var thead=document.querySelector('.matrix table thead');
if(!thead)return;
var wrapper=document.querySelector('.matrix');
var table=wrapper.querySelector('table');
var header=document.querySelector('.site-header');
var floatWrap=document.createElement('div');
floatWrap.className='matrix-float-head';
var floatTable=document.createElement('table');
var floatThead=thead.cloneNode(true);
floatTable.appendChild(floatThead);
floatWrap.appendChild(floatTable);
document.body.appendChild(floatWrap);
var liveCells=thead.querySelectorAll('th');
var cloneCells=floatThead.querySelectorAll('th');
function sizeCols(){
var wrapRect=wrapper.getBoundingClientRect();
floatWrap.style.left=wrapRect.left+'px';
floatWrap.style.width=wrapRect.width+'px';
floatTable.style.width=table.getBoundingClientRect().width+'px';
for(var i=0;i<liveCells.length;i++){
cloneCells[i].style.width=liveCells[i].getBoundingClientRect().width+'px';
}
}
var ticking=false;
function update(){
ticking=false;
var headerH=header?header.getBoundingClientRect().height:0;
var tableRect=table.getBoundingClientRect();
var theadH=thead.getBoundingClientRect().height;
var show=tableRect.top<headerH&&tableRect.bottom>headerH+theadH;
if(show){
sizeCols();
floatWrap.style.top=headerH+'px';
floatWrap.style.display='block';
floatTable.style.transform='translateX(-'+wrapper.scrollLeft+'px)';
}else{
floatWrap.style.display='none';
}
}
function onScroll(){
if(!ticking){
ticking=true;
requestAnimationFrame(update);
}
}
window.addEventListener('scroll',onScroll,{passive:true});
window.addEventListener('resize',onScroll);
wrapper.addEventListener('scroll',onScroll,{passive:true});
update();
});"

#let make-head(m) = {
  html.elem("meta", attrs: (charset: "utf-8"))
  html.elem("meta", attrs: (name: "viewport", content: "width=device-width, initial-scale=1"))
  let t = m.at("title", default: SITE.title)
  html.elem("title")[#(if t == SITE.title { t } else { t + " — " + SITE.title })]
  let desc = m.at("summary", default: SITE.tagline)
  if desc != none { html.elem("meta", attrs: (name: "description", content: desc)) }
  html.elem("link", attrs: (rel: "icon", href: "/assets/images/logo.svg", type: "image/svg+xml"))
  // Raster fallbacks for browsers/contexts that skip the svg favicon above,
  // rasterised at build time from that same logo.svg (www/generate.sh) — no
  // hand-maintained second copy of the mark.
  html.elem("link", attrs: (
    rel: "icon",
    href: "/assets/images/favicon-32.png",
    type: "image/png",
    sizes: "32x32",
  ))
  html.elem("link", attrs: (rel: "apple-touch-icon", href: "/assets/images/apple-touch-icon.png", sizes: "180x180"))
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
  html.script(_matrix-float-head-script)
  html.script(_search-init)
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
    html.elem("div", attrs: (class: "header-inner"), {
      _brand
      // Pagefind mounts its input here; the results drawer floats as a popup.
      html.elem("div", attrs: (class: "search", id: "search"))
      html.elem("div", attrs: (class: "header-right"), {
        html.elem("a", attrs: (href: REPO, class: "header-link"), [GitHub])
        _toggle
      })
    })
  })

  html.elem("div", attrs: (class: "shell"), {
    _sidebar(active)
    // data-pagefind-body scopes the search index to page content only, so the
    // header, sidebar nav, and footer never leak into results.
    html.elem("main", attrs: (class: if home { "home" } else { "" }, "data-pagefind-body": ""), {
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
