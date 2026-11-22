#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Replace the home hero's install line with an Install button",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  kind: "docs",
  tags: ("www", "docs", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the home page's install route matches the Quickstart heading it names],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ 🔴 Below the two hero buttons sits a monospace install line: `<p class="install">INSTALL <code>cargo install mindtape-cli</code></p>`.
+ 🔴 The line reads as a footnote, not a way in, and it is the page's only install route.


== TO-BE

- 🟢 `.hero-cta` holds three buttons in order: `Get started` (primary), `How it works` (ghost), `Install` (ghost).
- 🟢 `Install` links to `u("/quickstart/#install")`, the Quickstart's `Install` heading, which renders as `<h2 id="install">`.
- 🟢 No `.install` markup or CSS remains.
- 🟢 The Markdown copy's line reads `Get started · How it works · Install`.

```html
<div class="hero-cta"><a class="btn" href="…/quickstart">Get started <span>→</span></a><a class="btn btn-ghost" href="…/concepts">How it works</a><a class="btn btn-ghost" href="…/quickstart/#install">Install</a></div>
```


== Source of truth

- `www/utils/ui.typ`: `_hero-copy` builds `.hero-cta` and `.install`, and `_md-hero` builds the Markdown line.
- `www/content/quickstart.typ`: the `= Install` heading, which renders as `<h2 id="install">`.
- `www/assets/styles/components.css`: `.hero-cta` and `.install`.


== Changes

- `www/utils/ui.typ`: delete the `.install` paragraph and the unused `cli-crate` import, append the ghost `Install` anchor inside `.hero-cta`, and add the same link to `_md-hero`.
- `www/assets/styles/components.css`: delete `.install` and `.install code`, and make `.hero-cta`'s bottom margin `0` because nothing follows it.


== Out of scope

- The Quickstart page's own `Install` section and README's `Install` section both keep the command.
- The kicker, title, lede and pitch keep their copy.
- `DESIGN.md` already describes a ghost button under Buttons, and it does not mention the removed line.


== Dogfood

This repo's docs site builds from `www/`, so `just docs::check` regenerates the home page and fails before the new button lands.

