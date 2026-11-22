// Page chrome: <head>, masthead, route strip, line-map sidebar, main, footer.

#import "../utils/site.typ": (
  NAV, NAV_CHILDREN, REPO, SITE, SITE_URL, md-copy-button, md-path, mindtape,
  page-meta, pkg-version, render-summary, u,
)

// Set data-theme before first paint (no flash): the stored choice, else the
// system's prefers-color-scheme. toggleTheme flips and stores it. The masthead
// switch is parsed after this script, so its aria-checked follows the theme
// on DOMContentLoaded and on every toggle.
#let _theme-script = "(function(){var d=document.documentElement;var t=localStorage.getItem('theme');if(t){d.setAttribute('data-theme',t)}else if(window.matchMedia('(prefers-color-scheme:dark)').matches){d.setAttribute('data-theme','dark')}}());function syncThemeSwitch(){var dark=document.documentElement.getAttribute('data-theme')==='dark';document.querySelectorAll('.theme-switch').forEach(function(b){b.setAttribute('aria-checked',dark?'true':'false')})}function toggleTheme(){var d=document.documentElement;var n=d.getAttribute('data-theme')==='dark'?'light':'dark';d.setAttribute('data-theme',n);localStorage.setItem('theme',n);syncThemeSwitch()}document.addEventListener('DOMContentLoaded',syncThemeSwitch);"

// The narrow-screen nav drawer. Below main.css's 52rem breakpoint the line map
// becomes a drawer that the masthead's menu button opens. The `js-nav` class,
// set before first paint, turns the drawer on, so without script the sidebar
// stays in the page, after the content. Opening sets `nav-open` on <html>,
// pins the drawer under the masthead and scrolls it to the current page. A tap
// on a link or outside the drawer closes it, and so does Escape, which hands
// focus back to the button.
#let _nav-script = "document.documentElement.classList.add('js-nav');
function scrollNav(s,a){if(!s||!a)return;var r=s.getBoundingClientRect(),ar=a.getBoundingClientRect();if(ar.top>=r.top+16&&ar.bottom<=r.bottom-16)return;s.scrollTop+=ar.top-r.top-s.clientHeight/3;}
function setNav(open){
var d=document.documentElement,b=document.querySelector('.nav-toggle'),s=document.querySelector('.sidebar');
d.classList.toggle('nav-open',open);
if(b)b.setAttribute('aria-expanded',open?'true':'false');
if(!open||!s)return;
d.style.setProperty('--header-bottom',document.querySelector('.masthead').getBoundingClientRect().bottom+'px');
scrollNav(s,s.querySelector('[aria-current=page]'));
}
document.addEventListener('click',function(e){
var open=document.documentElement.classList.contains('nav-open');
if(e.target.closest('.nav-toggle'))setNav(!open);
else if(open&&(e.target.closest('.sidebar a')||!e.target.closest('.sidebar')))setNav(false);
});
document.addEventListener('keydown',function(e){
if(e.key!=='Escape'||!document.documentElement.classList.contains('nav-open'))return;
setNav(false);document.querySelector('.nav-toggle').focus();
});
document.addEventListener('DOMContentLoaded',function(){
var s=document.querySelector('.sidebar');if(!s||s.scrollHeight<=s.clientHeight)return;
scrollNav(s,s.querySelector('[aria-current=page]'));
});
document.addEventListener('DOMContentLoaded',function(){
document.querySelectorAll('.nav-group.is-here').forEach(function(g){
var a=g.querySelector('.stop.is-current');if(!a)return;
var gr=g.getBoundingClientRect(),ar=a.getBoundingClientRect();
g.style.setProperty('--here',Math.max(0,ar.top+ar.height/2-gr.top-4)+'px');
requestAnimationFrame(function(){g.classList.add('line-drawn')});
});
});"

// Bootstrap Pagefind's search UI. The bundle (UI JS/CSS + index) is generated
// post-build by `pagefind --site public/mindtape` (.just/docs.just's `build`)
// into /mindtape/pagefind/, which is invisible to `tola build`/`validate`.
// Injecting the stylesheet + script at runtime (rather than static
// <link>/<script>) keeps tola's asset validator, which only knows build-time
// assets, from failing on them. Once the script loads, mount PagefindUI into
// the masthead #search box (waiting for the DOM if the async script wins the
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
    + "new PagefindUI({element:'#search',showSubResults:true,showImages:false,bundlePath:p,translations:{placeholder:'Search documentation'}});"
    + "var box=document.getElementById('search');if(!box)return;"
    + "document.addEventListener('click',function(e){if(!box.contains(e.target)&&!e.target.closest('.search-key')){box.classList.add('search--closed');}});"
    + "box.addEventListener('focusin',function(){box.classList.remove('search--closed');});"
    + "document.addEventListener('keydown',function(e){if(e.key==='/'){e.preventDefault();var i=box.querySelector('input');if(i)i.focus();}if(e.key==='Escape'){box.classList.add('search--closed');box.blur&&box.blur();}});"
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
var header=document.querySelector('.masthead');
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

#let _code-panel-script = "document.addEventListener('DOMContentLoaded',function(){
var names={sh:'SHELL',bash:'SHELL',typ:'TYPST',toml:'TOML',nix:'NIX',json:'JSON',rust:'RUST',js:'JS',javascript:'JS',html:'HTML',css:'CSS',yaml:'YAML',text:'TEXT'};
document.querySelectorAll('main pre').forEach(function(pre){
if(pre.closest('.code')||pre.closest('.terminal')||pre.closest('.lsp'))return;
var code=pre.querySelector('code');
var lang=code&&code.getAttribute('data-lang');
var box=document.createElement('div');box.className='code';
var head=document.createElement('div');head.className='code-head';
var sq=document.createElement('span');sq.className='sq';
var label=document.createElement('span');label.className='code-lang';
label.textContent=names[lang]||'CODE';
var btn=document.createElement('button');btn.className='copy';btn.type='button';btn.textContent='COPY';btn.setAttribute('aria-label','Copy code');
head.appendChild(sq);head.appendChild(label);head.appendChild(btn);
pre.parentNode.insertBefore(box,pre);box.appendChild(head);box.appendChild(pre);
});
});"

#let _code-copy-script = "document.addEventListener('click',function(e){
var b=e.target.closest('.copy');if(!b)return;
var box=b.closest('.code');if(!box)return;
var pre=box.querySelector('pre');if(!pre)return;
var done=navigator.clipboard&&navigator.clipboard.writeText
?navigator.clipboard.writeText(pre.innerText)
:Promise.reject();
done.then(function(){var was=b.textContent;b.textContent='COPIED';setTimeout(function(){b.textContent=was},1200)});
});"

// "Calling points", the article's on-this-page index. The headings themselves
// are the only source: the script reads the rendered h2 ids that base.typ's
// slug rule gives them, so the rail cannot drift from the page. On a wide
// screen the list fills the right rail beside the prose; below the breakpoint
// the rail is hidden and the same list becomes a folded box above the first
// heading, which is where a phone reader needs it. A page with fewer than
// three headings gets no index at all.
//
// The rail tracks the heading under the masthead as the reader scrolls. Every
// stop above the current one is passed, and a green line runs from the list
// top to the current dot, whose centre the script writes to `--toc-here`. A
// click holds the mark on that stop through the smooth scroll, and the next
// wheel, touch or key hands it back to the reading position. A page opened
// at a fragment marks that section on the first frame.
#let _toc-script = "document.addEventListener('DOMContentLoaded',function(){
var main=document.querySelector('main[data-pagefind-body]');
if(!main)return;
var heads=main.querySelectorAll('h2[id]');
if(heads.length<3)return;
var rail=main.querySelector('.page-rail');
var narrow=window.matchMedia('(max-width:52rem)').matches;
var flow=narrow||!rail;
var list=document.createElement('ul');
heads.forEach(function(h){
var li=document.createElement('li');
var a=document.createElement('a');
a.href='#'+h.id;
a.textContent=h.textContent;
li.appendChild(a);
list.appendChild(li);
});
var nav=document.createElement('nav');
nav.className='page-toc';
nav.setAttribute('data-pagefind-ignore','');
if(flow){
nav.className='page-toc page-toc-flow';
var details=document.createElement('details');
details.open=!narrow;
var summary=document.createElement('summary');
summary.textContent='Calling points';
details.appendChild(summary);
details.appendChild(list);
nav.appendChild(details);
heads[0].parentNode.insertBefore(nav,heads[0]);
}else{
var eyebrow=document.createElement('p');
eyebrow.className='eyebrow';
eyebrow.textContent='Calling points';
nav.appendChild(eyebrow);
nav.appendChild(list);
rail.insertBefore(nav,rail.firstChild);
}
nav.classList.add('page-toc-live');
var items=Array.prototype.slice.call(list.children);
var current=-1,locked=-1,frame=0;
function dotCenter(i){
var host=flow?items[i].firstChild:items[i];
var hr=host.getBoundingClientRect(),lr=list.getBoundingClientRect();
return hr.top-lr.top+hr.height/2;
}
function setCurrent(i){
if(i!==current){
current=i;
items.forEach(function(li,k){
li.classList.toggle('is-passed',k<i);
li.classList.toggle('is-current',k===i);
var a=li.firstChild;
if(k===i)a.setAttribute('aria-current','location');
else a.removeAttribute('aria-current');
});
}
list.style.setProperty('--toc-here',dotCenter(i)+'px');
}
function pick(){
var line=96,i=0;
for(var k=0;k<heads.length;k++){
if(heads[k].getBoundingClientRect().top<=line)i=k;else break;
}
if(window.innerHeight+window.pageYOffset>=document.documentElement.scrollHeight-2)i=heads.length-1;
return i;
}
function update(){
frame=0;
setCurrent(locked>=0?locked:pick());
}
function queue(){
if(!frame)frame=requestAnimationFrame(update);
}
function release(){
if(locked<0)return;
locked=-1;
queue();
}
list.addEventListener('click',function(e){
var a=e.target.closest('a');
if(!a||e.metaKey||e.ctrlKey||e.shiftKey||e.altKey)return;
var i=items.indexOf(a.parentNode);
if(i<0)return;
locked=i;
setCurrent(i);
});
['wheel','touchstart','keydown','pointerdown'].forEach(function(type){
window.addEventListener(type,release,{passive:true});
});
window.addEventListener('scroll',queue,{passive:true});
window.addEventListener('resize',queue);
window.addEventListener('load',queue);
var box=nav.querySelector('details');
if(box)box.addEventListener('toggle',queue);
update();
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
  // rasterised at build time from that same logo.svg (www/generate.sh), so no
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
  // The faces are self-hosted (assets/fonts/, @font-face in main.css), so the
  // page makes no third-party request. Preload the face above the fold; the
  // CSS load would otherwise wait on HTML parsing first.
  html.elem("link", attrs: (
    rel: "preload",
    href: "/assets/fonts/archivo-latin.woff2",
    "as": "font",
    type: "font/woff2",
    crossorigin: "anonymous",
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
  html.script(_code-panel-script)
  html.script(_code-copy-script)
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
    + html.elem("span", attrs: (class: "brand-name"), [#mindtape])
    + html.elem("span", attrs: (class: "badge"), "DOCS"),
)

// An inline 24-unit SVG icon drawn in currentColor, so the masthead's CSS
// colors it for either theme. `stroke: true` draws the paths as 2-unit lines,
// the way the sun and moon are drawn; otherwise they are filled.
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
    class: "tool header-icon",
    "aria-label": "GitHub",
    title: "GitHub",
  ),
  html.elem("span", attrs: (class: "icon github-mark")),
)

// The theme switch: a squared track whose knob slides right for dark mode and
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
    class: "tool nav-toggle",
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

// ── Route helpers ───────────────────────────────────────────────────────────
// The docs are one line, and each nav group is a numbered stretch of it. These
// helpers read the NAV tree rather than restating its order, so the route
// strip, the sidebar and an article's kicker always agree.

#let _two(n) = if n < 10 { "0" + str(n) } else { str(n) }

#let _group-index(route) = {
  for (i, group) in NAV.enumerate() {
    for item in group.items {
      if item == route or (item != "/" and route.starts-with(item)) {
        return i
      }
    }
  }
  0
}

#let _group-at(index) = NAV.at(index)

// Every page in reading order, with a nav entry's children after it: the
// article footer's next stop and the rail's page count both use this.
#let _reading-order = NAV
.map(group => group.items.map(route => {
if route in NAV_CHILDREN { (route,) + NAV_CHILDREN.at(route) } else {
(route,)
}
}))
.flatten()

#let _next-route(active) = {
  let at = _reading-order.position(r => r == active)
  if at == none or at + 1 >= _reading-order.len() { none } else {
    _reading-order.at(at + 1)
  }
}

// A page's number on its own stretch of the line: 01 is the group's first
// page, 02 its second, and a nav entry's children follow it. The kicker's
// number is this page number, while the sidebar's is the group's.
#let _page-no(active, here) = {
  let group = _group-at(here)
  let order = group
    .items
    .map(route => if route in NAV_CHILDREN {
      (route,) + NAV_CHILDREN.at(route)
    } else { (route,) })
    .flatten()
  let at = order.position(r => r == active)
  if at == none { here + 1 } else { at + 1 }
}

// One sidebar stop. `class` is "stop" for a page and "stop sub" for a CLI
// command under its index. The current page fills its dot; a parent of the
// current page keeps its dot outlined and gets aria-current="true", matching
// the old sidebar's behavior for nested routes.
#let _nav-link(route, active, class: "stop") = {
  let meta = page-meta(route)
  if meta == none { return none }
  let cls = class
  if active == route { cls = cls + " is-current" }
  let attrs = (href: u(route), class: cls)
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
    html.elem("ul", attrs: (class: "sub-stops"), for kid in kids {
      let kid-link = _nav-link(kid, active, class: "stop sub")
      if kid-link != none { html.elem("li", kid-link) }
    })
  }))
}

// The line map: numbered groups, a rule down each group, and a dot stop per
// page. The group holding the current page runs green.
#let _sidebar(active, here) = html.elem(
  "aside",
  attrs: (class: "sidebar", id: "site-nav"),
  {
    let current = page-meta(active)
    html.elem("div", attrs: (class: "rail-head"), {
      html.elem("span", attrs: (class: "eyebrow"), "Documentation")
      if current != none {
        html.elem("strong", [#current.label])
      }
    })
    html.elem("nav", attrs: ("aria-label": "Documentation"), {
      for (i, group) in NAV.enumerate() {
        html.elem(
          "div",
          attrs: (class: "nav-group" + if i == here { " is-here" } else { "" }),
          {
            html.elem("p", attrs: (class: "nav-title"), {
              html.elem("span", attrs: (class: "group-no"), _two(i + 1))
              group.title
            })
            html.elem("ul", for route in group.items {
              _nav-item(route, active)
            })
          },
        )
      }
    })
    html.elem("div", attrs: (class: "rail-foot"), {
      html.elem("span", "v" + pkg-version)
      html.elem("span", "MIT")
    })
  },
)

// The seven stations. A station is a nav group; the current page lights the
// station its group owns.
#let _route-strip(active, here) = html.elem(
  "nav",
  attrs: (class: "line", "aria-label": "Documentation sections"),
  html.elem("div", attrs: (class: "wrap line-inner"), {
    html.elem("span", attrs: (class: "line-no"), "Line 01")
    for (i, group) in NAV.enumerate() {
      html.elem(
        "a",
        attrs: (
          href: u(group.items.at(0)),
          class: "station" + if i == here { " is-active" } else { "" },
        ),
        {
          html.elem("span", attrs: (class: "dot"))
          group.title
        },
      )
    }
    html.elem("span", attrs: (class: "line-end"), "MIND TAPE / LOCAL")
  }),
)

// A page's kicker: its group number, the group and the page, in the transit
// style. `bar` draws the hairline that runs to the hero's right edge.
#let _kicker(active, here, bar: false) = {
  let group = _group-at(here)
  let meta = page-meta(active)
  html.elem("p", attrs: (class: "kicker"), {
    html.elem("span", attrs: (class: "num"), _two(_page-no(active, here)))
    " "
    (
      upper(group.title)
        + " / "
        + if meta != none { upper(meta.label) } else { "" }
    )
    if bar { html.elem("span", attrs: (class: "bar")) }
  })
}

// The rail beside an article: the page's line, its group, the group's page
// count and the package version. The calling-points list is inserted above
// this block at runtime.
#let _page-rail(active, here) = {
  let group = _group-at(here)
  html.elem(
    "aside",
    attrs: (class: "page-rail", "data-pagefind-ignore": ""),
    html.elem("div", attrs: (class: "rail-meta"), {
      html.elem("span")[Line 01 / #upper(group.title)]
      html.elem("span")[#str(group.items.len()) PAGES]
      html.elem("span")[Version #pkg-version]
    }),
  )
}

// NEXT STOP: the next page in reading order, so an article ends at the line
// rather than at a dead end. The last page points back to the start.
#let _next-stop(active) = {
  let next = _next-route(active)
  let target = if next == none { "/" } else { next }
  let meta = page-meta(target)
  if meta == none { return none }
  let group = _group-at(_group-index(target))
  html.elem("a", attrs: (class: "next-stop", href: u(target)), {
    html.elem("span", attrs: (class: "next-kicker"), "Next stop")
    html.elem("span", attrs: (class: "next-route"), {
      _two(_group-index(target) + 1) + " " + upper(group.title)
    })
    html.elem("span", attrs: (class: "next-label"), [#meta.label])
    html.elem("span", attrs: (class: "next-arrow"), "→")
  })
}

#let layout(body, meta: (:)) = {
  let title = meta.at("title", default: none)
  let active = meta.at("active", default: none)
  let home = meta.at("home", default: false)
  // The lede and make-head's <meta name="description"> both render the
  // page's own summary through `render-summary`.
  let summary = render-summary(meta.at("summary", default: none))
  let here = _group-index(active)

  html.elem("header", attrs: (class: "masthead"), {
    html.elem("div", attrs: (class: "wrap masthead-inner"), {
      html.elem("a", attrs: (class: "skip", href: "#main"), "Skip to content")
      _brand
      // Pagefind mounts its input here; the results drawer floats as a popup.
      // The static field underneath keeps the search affordance visible before
      // the index loads, and on `tola serve`, which has no index. Its icon and
      // label are drawn in CSS so the flow holds no block inside an inline.
      html.elem("div", attrs: (class: "search-wrap"), {
        html.elem("span", attrs: (
          class: "search-static",
          "aria-hidden": "true",
        ))
        html.elem("div", attrs: (class: "search", id: "search"))
        html.elem(
          "span",
          attrs: (class: "search-key", "aria-hidden": "true"),
          "/",
        )
      })
      html.elem("div", attrs: (class: "tools"), {
        _github
        _toggle
        _menu
      })
    })
  })

  _route-strip(active, here)

  html.elem(
    "div",
    attrs: (class: "wrap shell" + if home { " shell-home" } else { "" }),
    {
      _sidebar(active, here)
      // data-pagefind-body scopes the search index to page content only, so the
      // masthead, sidebar nav, and footer never leak into results.
      html.elem(
        "main",
        attrs: (
          id: "main",
          class: if home { "home" } else { "" },
          "data-pagefind-body": "",
        ),
        {
          if title != none and not home {
            html.elem("div", attrs: (class: "article-head"), {
              _kicker(active, here)
              html.elem("h1", title)
              html.elem("div", attrs: (class: "article-lede"), {
                if summary != none {
                  html.elem("p", attrs: (class: "lede"))[#summary]
                }
                md-copy-button(active)
              })
            })
            html.elem("div", attrs: (class: "article-body"), {
              html.elem("div", attrs: (class: "prose"), {
                body
                _next-stop(active)
              })
              _page-rail(active, here)
            })
          } else {
            body
          }
        },
      )
    },
  )

  html.elem("footer", attrs: (class: "site-footer"), {
    html.elem("div", attrs: (class: "wrap footer-inner"), {
      html.elem("span", attrs: (class: "end"), {
        html.elem("i")
        "End of line 01"
      })
      html.elem("span", "MIND TAPE · MIT · 2026")
      html.elem("span", attrs: (class: "right"), {
        link(REPO)[GitHub]
        link(u("/quickstart/"))[Quickstart]
        link(_md-url(active))[Markdown]
      })
    })
  })
}
