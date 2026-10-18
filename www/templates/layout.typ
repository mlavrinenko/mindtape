// Page chrome: <head>, header, docs sidebar, main, footer.

#import "../utils/site.typ": (
  NAV, NAV_CHILDREN, REPO, SITE, SITE_URL, md-copy-button, md-path, mindtape,
  page-meta, render-summary, u,
)

// Set data-theme before first paint (no flash): the stored choice, else the
// system's prefers-color-scheme. toggleTheme flips and stores it. The header
// switch is parsed after this script, so its aria-checked follows the theme
// on DOMContentLoaded and on every toggle.
#let _theme-script = "(function(){var d=document.documentElement;var t=localStorage.getItem('theme');if(t){d.setAttribute('data-theme',t)}else if(window.matchMedia('(prefers-color-scheme:dark)').matches){d.setAttribute('data-theme','dark')}}());function syncThemeSwitch(){var dark=document.documentElement.getAttribute('data-theme')==='dark';document.querySelectorAll('.theme-switch').forEach(function(b){b.setAttribute('aria-checked',dark?'true':'false')})}function toggleTheme(){var d=document.documentElement;var n=d.getAttribute('data-theme')==='dark'?'light':'dark';d.setAttribute('data-theme',n);localStorage.setItem('theme',n);syncThemeSwitch()}document.addEventListener('DOMContentLoaded',syncThemeSwitch);"

// The narrow-screen nav drawer. Below main.css's 52rem breakpoint the sidebar
// is a drawer that the header's menu button opens. The `js-nav` class, set
// before first paint, turns the drawer on, so without script the sidebar
// stays in the page, after the content. Opening sets `nav-open` on <html>,
// pins the drawer under the header and scrolls it to the current page. A tap
// on a link or outside the drawer closes it, and so does Escape, which hands
// focus back to the button.
#let _nav-script = "document.documentElement.classList.add('js-nav');
function setNav(open){
var d=document.documentElement,b=document.querySelector('.nav-toggle'),s=document.querySelector('.sidebar');
d.classList.toggle('nav-open',open);
if(b)b.setAttribute('aria-expanded',open?'true':'false');
if(!open||!s)return;
d.style.setProperty('--header-bottom',document.querySelector('.site-header').getBoundingClientRect().bottom+'px');
var a=s.querySelector('[aria-current=page]');
s.scrollTop=a?a.offsetTop-s.clientHeight/3:0;
}
document.addEventListener('click',function(e){
var open=document.documentElement.classList.contains('nav-open');
if(e.target.closest('.nav-toggle'))setNav(!open);
else if(open&&(e.target.closest('.sidebar a')||!e.target.closest('.sidebar')))setNav(false);
});
document.addEventListener('keydown',function(e){
if(e.key!=='Escape'||!document.documentElement.classList.contains('nav-open'))return;
setNav(false);document.querySelector('.nav-toggle').focus();
});"

// Bootstrap Pagefind's search UI. The bundle (UI JS/CSS + index) is generated
// post-build by `pagefind --site public/mindtape` (.just/docs.just's `build`)
// into /mindtape/pagefind/, which is invisible to `tola build`/`validate`.
// Injecting the stylesheet + script at runtime (rather than static
// <link>/<script>) keeps tola's asset validator — which only knows build-time
// assets — from failing on them. Once the script loads, mount PagefindUI into
// the header #search box (waiting for the DOM if the async script wins the
// race), then make the results drawer behave like a popup: dismiss on
// outside-click / Escape via a `search--closed` class the CSS honors, and
// re-open when the input regains focus. Search is a production-build feature;
// `tola serve` has no index.
#let _search-init = (
  "(function(){var p='"
    + u("/pagefind/")
    + "';"
    + "var l=document.createElement('link');l.rel='stylesheet';l.href=p+'pagefind-ui.css';document.head.appendChild(l);"
    + "var s=document.createElement('script');s.src=p+'pagefind-ui.js';s.onload=function(){var init=function(){"
    + "new PagefindUI({element:'#search',showSubResults:true,showImages:false,bundlePath:p});"
    + "var box=document.getElementById('search');if(!box)return;"
    + "document.addEventListener('click',function(e){if(!box.contains(e.target)){box.classList.add('search--closed');}});"
    + "box.addEventListener('focusin',function(){box.classList.remove('search--closed');});"
    + "document.addEventListener('keydown',function(e){if(e.key==='Escape'){box.classList.add('search--closed');box.blur&&box.blur();}});"
    + "};if(document.readyState!=='loading'){init();}else{document.addEventListener('DOMContentLoaded',init);}};document.head.appendChild(s);}());"
)

// Two aids for wide tables like the comparison matrix, set up on
// DOMContentLoaded because this script tag sits in <head>.
//
// Each `.matrix` scroll container marks its `.matrix-frame` with
// `matrix-more-left` and `matrix-more-right` while it can scroll that way,
// and components.css draws a shadow on each marked edge. The marks follow
// every scroll and resize.
//
// A floating <thead> clone. Plain CSS `position: sticky` on `thead th` can't
// work there: the `.matrix` wrapper is a horizontal scroll container
// (`overflow-x: auto`), and that clips block-axis sticky positioning for its
// descendants. No-op on pages without a `.matrix table thead`.
#let _matrix-script = "document.addEventListener('DOMContentLoaded',function(){
document.querySelectorAll('.matrix').forEach(function(box){
var frame=box.parentNode;
function edges(){
var max=box.scrollWidth-box.clientWidth;
frame.classList.toggle('matrix-more-left',box.scrollLeft>1);
frame.classList.toggle('matrix-more-right',box.scrollLeft<max-1);
}
box.addEventListener('scroll',edges,{passive:true});
window.addEventListener('resize',edges);
edges();
});
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

// "On this page" index, derived at runtime from the rendered h2 elements (see
// base.typ's slug() show rule for their ids) — never a hand-written list, so
// it cannot drift from the content it indexes. Only worth showing once a page
// clears a minimum of 3 headings; a shorter page skims fine without one. The
// `open` state is decided once, before the element is inserted, from the same
// 52rem breakpoint main.css uses for its narrow layout: expanded on wide
// viewports, collapsed (a `<details>` toggle) on narrow ones so it never
// shoves the content down. `data-pagefind-ignore` keeps this nav out of the
// search index, matching the header/sidebar/footer it sits alongside.
// Copy page as Markdown: fetch the page's Markdown copy and put it on the
// clipboard. ClipboardItem takes the fetch as a promise, so Safari keeps the
// click's permission while the fetch runs. If the copy fails, the copy opens
// in a new tab instead.
#let _md-copy-script = "document.addEventListener('click',function(e){
var b=e.target.closest('.md-copy');if(!b)return;
var url=b.getAttribute('data-md'),tip=b.getAttribute('data-tip');
var text=fetch(url).then(function(r){if(!r.ok)throw new Error(r.status);return r.text()});
var done=window.ClipboardItem&&navigator.clipboard&&navigator.clipboard.write
?navigator.clipboard.write([new ClipboardItem({'text/plain':text.then(function(t){return new Blob([t],{type:'text/plain'})})})])
:text.then(function(t){return navigator.clipboard.writeText(t)});
done.then(function(){
b.setAttribute('data-tip','Copied');b.classList.add('md-copy-said');
setTimeout(function(){b.setAttribute('data-tip',tip);b.classList.remove('md-copy-said')},1500);
},function(){window.open(url,'_blank')});
});"

#let _toc-script = "document.addEventListener('DOMContentLoaded',function(){
var main=document.querySelector('main[data-pagefind-body]');
if(!main)return;
var heads=main.querySelectorAll('h2[id]');
if(heads.length<3)return;
var nav=document.createElement('nav');
nav.className='page-toc';
nav.setAttribute('data-pagefind-ignore','');
var details=document.createElement('details');
details.open=window.matchMedia('(min-width:52.01rem)').matches;
var summary=document.createElement('summary');
summary.textContent='On this page';
var list=document.createElement('ul');
heads.forEach(function(h){
var li=document.createElement('li');
var a=document.createElement('a');
a.href='#'+h.id;
a.textContent=h.textContent;
li.appendChild(a);
list.appendChild(li);
});
details.appendChild(summary);
details.appendChild(list);
nav.appendChild(details);
heads[0].parentNode.insertBefore(nav,heads[0]);
});"

// The page's Markdown copy, which www/generate-md.sh writes beside its HTML.
// The URL is absolute because `tola validate` resolves a site path against
// its own page list, which holds no `.md` file.
#let _md-url(route) = SITE_URL + md-path(route)

#let make-head(m) = {
  html.elem("meta", attrs: (charset: "utf-8"))
  html.elem("meta", attrs: (
    name: "viewport",
    content: "width=device-width, initial-scale=1",
  ))
  let t = m.at("title", default: SITE.title)
  html.elem("title")[#(
    if t == SITE.title { t } else { t + " — " + SITE.title }
  )]
  let desc = render-summary(m.at("summary", default: none), plain: true)
  html.elem("meta", attrs: (
    name: "description",
    content: if desc == none { SITE.tagline } else { desc },
  ))
  html.elem("link", attrs: (
    rel: "icon",
    href: "/assets/images/logo.svg",
    type: "image/svg+xml",
  ))
  // Raster fallbacks for browsers/contexts that skip the svg favicon above,
  // rasterised at build time from that same logo.svg (www/generate.sh) — no
  // hand-maintained second copy of the mark.
  html.elem("link", attrs: (
    rel: "icon",
    href: "/assets/images/favicon-32.png",
    type: "image/png",
    sizes: "32x32",
  ))
  html.elem("link", attrs: (
    rel: "apple-touch-icon",
    href: "/assets/images/apple-touch-icon.png",
    sizes: "180x180",
  ))
  html.elem("link", attrs: (
    rel: "preconnect",
    href: "https://fonts.googleapis.com",
  ))
  html.elem("link", attrs: (
    rel: "preconnect",
    href: "https://fonts.gstatic.com",
    crossorigin: "",
  ))
  html.elem("link", attrs: (
    rel: "stylesheet",
    href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&family=Source+Serif+4:opsz,wght@8..60,400;8..60,600;8..60,700&display=swap",
  ))
  html.elem("link", attrs: (
    rel: "alternate",
    type: "text/markdown",
    href: _md-url(m.active),
  ))
  // Assets stay source-relative; tola applies the site prefix on build.
  html.elem("link", attrs: (rel: "stylesheet", href: "/assets/styles/main.css"))
  html.elem("link", attrs: (
    rel: "stylesheet",
    href: "/assets/styles/components.css",
  ))
  html.script(_theme-script)
  html.script(_nav-script)
  html.script(_matrix-script)
  html.script(_toc-script)
  html.script(_md-copy-script)
  html.script(_search-init)
}

#let _brand = html.elem(
  "a",
  attrs: (href: u("/"), class: "brand"),
  // Logo as an accent-tinted inline mark (see .brand-mark in main.css); a real
  // block-level <img> would force a <p> inside the <a> and split the anchor.
  html.elem("span", attrs: (
    class: "brand-mark",
    role: "img",
    "aria-label": mindtape,
  ))
    + html.elem("span", attrs: (class: "brand-name"), [#mindtape]),
)

// An inline 24-unit SVG icon drawn in currentColor, so the header's CSS colors
// it for either theme. `stroke: true` draws the paths as 2-unit lines, the
// way the sun and moon are drawn; otherwise they are filled.
#let _icon(class, ..paths, stroke: false) = html.elem(
  "svg",
  attrs: (
    class: class,
    viewBox: if stroke { "0 0 24 24" } else { "0 0 16 16" },
    "aria-hidden": "true",
    focusable: "false",
  )
    + if stroke {
      (
        fill: "none",
        stroke: "currentColor",
        "stroke-width": "2",
        "stroke-linecap": "round",
        "stroke-linejoin": "round",
      )
    } else { (fill: "currentColor") },
  paths.pos().map(d => html.elem("path", attrs: (d: d))).join(),
)

// The GitHub mark (Octicons' mark-github, 16 units).
#let _github = html.elem(
  "a",
  attrs: (
    href: REPO,
    class: "header-icon",
    "aria-label": "GitHub",
    title: "GitHub",
  ),
  _icon(
    "icon",
    "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z",
  ),
)

// The theme switch: a pill track whose knob slides right for dark mode and
// shows the sun or the moon. It is a native button, so Enter and Space flip
// it, and role="switch" with aria-checked tells a screen reader whether dark
// mode is on. The theme script keeps aria-checked in step.
#let _toggle = html.elem(
  "button",
  attrs: (
    class: "theme-switch",
    type: "button",
    role: "switch",
    "aria-checked": "false",
    "aria-label": "Dark mode",
    title: "Dark mode",
    onclick: "toggleTheme()",
  ),
  html.elem("span", attrs: (class: "theme-knob"), {
    _icon(
      "icon theme-sun",
      "M12 8a4 4 0 1 0 0 8a4 4 0 1 0 0-8z",
      "M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41",
      stroke: true,
    )
    _icon(
      "icon theme-moon",
      "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z",
      stroke: true,
    )
  }),
)

// The menu button, shown below the 52rem breakpoint once the nav script runs.
// It shows three bars while the drawer is closed and a cross while it is
// open, and aria-expanded tells a screen reader which.
#let _menu = html.elem(
  "button",
  attrs: (
    class: "header-icon nav-toggle",
    type: "button",
    "aria-label": "Menu",
    "aria-expanded": "false",
    "aria-controls": "site-nav",
    title: "Menu",
  ),
  {
    _icon("icon nav-bars", "M4 6h16M4 12h16M4 18h16", stroke: true)
    _icon("icon nav-cross", "M6 6l12 12M18 6L6 18", stroke: true)
  },
)

// One sidebar link. `page-meta` returns `none` only during
// www/generate-site-pages.sh's bootstrap pass, before the manifest it queries
// every page to build exists yet, so that entry renders nothing rather than a
// labelless link. Once the manifest is real, a route with none in it is a
// bug, not a skip (see page-meta's own comment), so nothing here swallows
// that case. A nested page such as /cli/ls/ is the current page in its own
// entry and marks its parent's entry as the current section.
#let _nav-link(route, active) = {
  let meta = page-meta(route)
  if meta == none { return none }
  let attrs = (href: u(route))
  if active == route {
    attrs.insert("aria-current", "page")
  } else if (
    route != "/" and type(active) == str and active.starts-with(route)
  ) {
    attrs.insert("aria-current", "true")
  }
  html.elem("a", attrs: attrs, [#meta.label])
}

// An entry with child pages is a native <details>, so it needs no script. It
// is open on the entry's own page and on every page under it, and the
// reader's toggle holds until the next navigation. The entry's link sits in
// the <summary>, so one row both opens the list and goes to the entry's page.
#let _nav-item(route, active) = {
  let kids = NAV_CHILDREN.at(route, default: ())
  let link = _nav-link(route, active)
  if link == none { return none }
  if kids.len() == 0 { return html.elem("li", link) }
  let attrs = (class: "nav-nested")
  if type(active) == str and active.starts-with(route) {
    attrs.insert("open", "")
  }
  html.elem("li", html.elem("details", attrs: attrs, {
    html.elem("summary", link)
    html.elem("ul", for kid in kids {
      let kid-link = _nav-link(kid, active)
      if kid-link != none { html.elem("li", kid-link) }
    })
  }))
}

#let _sidebar(active) = html.elem(
  "aside",
  attrs: (class: "sidebar", id: "site-nav"),
  {
    html.elem("nav", attrs: ("aria-label": "Documentation"), {
      for group in NAV {
        html.elem("div", attrs: (class: "nav-group"), {
          html.elem("p", attrs: (class: "nav-title"), [#group.title])
          html.elem("ul", for route in group.items { _nav-item(route, active) })
        })
      }
    })
  },
)

#let layout(body, meta: (:)) = {
  let title = meta.at("title", default: none)
  let active = meta.at("active", default: none)
  let home = meta.at("home", default: false)
  // The lede and make-head's <meta name="description"> both render the
  // page's own summary through `render-summary`.
  let summary = render-summary(meta.at("summary", default: none))

  html.elem("header", attrs: (class: "site-header"), {
    html.elem("div", attrs: (class: "header-inner"), {
      _brand
      // Pagefind mounts its input here; the results drawer floats as a popup.
      html.elem("div", attrs: (class: "search", id: "search"))
      html.elem("div", attrs: (class: "header-right"), {
        _github
        _toggle
        _menu
      })
    })
  })

  html.elem("div", attrs: (class: "shell"), {
    _sidebar(active)
    // data-pagefind-body scopes the search index to page content only, so the
    // header, sidebar nav, and footer never leak into results.
    html.elem(
      "main",
      attrs: (class: if home { "home" } else { "" }, "data-pagefind-body": ""),
      {
        if title != none and not home {
          html.elem(
            "div",
            attrs: (class: "page-title"),
            html.elem("h1")[#title] + md-copy-button(active),
          )
        }
        // The home page carries its own hero lede (see hero() in ui.typ); skip
        // here so it never doubles up.
        if summary != none and not home {
          html.elem("p", attrs: (class: "lede"))[#summary]
        }
        body
      },
    )
  })

  html.elem("footer", attrs: (class: "site-footer"), {
    html.elem("p")[
      #SITE.title · #link(REPO)[GitHub] · MIT
    ]
  })
}
