#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Strip the decorative runs from comment banners",
  priority: ice(
    impact: 3,
    confidence: 1.0,
    ease: 10,
  ),
  difficulty: 1,
  kind: "refactor",
  tags: ("tooling", "stateless"),
  x-goals: (
    product: (
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  status: done(2026, 11, 13),
  assignee: "agent",
)

== AS-IS

+ Run `git grep -nE '[-=*_~#]{5,}$' -- '*.rs' '*.sh' '*.css'` before this change.
+ 🔴 It prints 129 comment lines, one per decorative section header.
+ Run `just check`; no gate names a new one.

```
$ sed -n 13p .just/scripts/_gate-drift-lib.sh
# arm 1 ----------------------------------------------------------------------
```


== TO-BE

- 🟢 Every comment keeps its words and loses the run.
- 🟢 The same `git grep` prints nothing.
- 🟢 The 40 touched files keep their tests, prose and behaviour.

```
# before
# arm 1 ----------------------------------------------------------------------
# after
# arm 1
```


== Changes

- `crates/**/*.rs`: strip the trailing run from 83 test and module comments.
- `www/**/*.sh`: strip 28 script section headers, including the two in `.just/scripts`.
- `www/assets/styles/*.css`: strip 18 stylesheet section headers.


== Out of scope

- The Vale gate that holds the tree clean; it waits for Vale 3.27.
- Typst comments: the only banners are in generated `tola.typ`.
- Markdown tables and `---` rules: they are syntax, not decoration.


== Dogfood

This repo's comments lose the runs, and the follow-up gate fails on a new one.

