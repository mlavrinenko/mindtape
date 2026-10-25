#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Skip inline code in a content density count",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "gating"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a density rule counts what the author wrote, not the commands they quote],
      enforced-practice: helps[a prose rule can hold without a false hit on every code span],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 25),
)

== Problem

+ Add a `density` content rule with `pattern = '\?(\s|$)'`, `max = 0` and `per = "sentence"` over a TO-BE section.
+ Write the bullet ``` - `mt ls ? --help` shows the flag. ``` there.
+ Run `mt check`.
+ 🔴 It fails on sentence 2, quoted as ``` ("- `mt ls ?") ```: the `?` inside the code span counts, and it splits the sentence in the middle of the span.


== TO-BE

- 🟢 A `density` assertion drops inline code as well as fenced blocks before it splits and counts, with no new config key.
- 🟢 A single-backtick span and a one-line span of three or more backticks both count as code. Prose outside the span still counts.
- 🟢 A sentence never splits inside a span, and the snippet a violation quotes is the original text, without the list marker.

```toml
assert = { density = { pattern = '\?(\s|$)', max = 0, per = "sentence" } }
```


== Changes

- `crates/mindtape/src/body/content_gate/content_gate_density_tests.rs`: the red tests first, a `?` inside single and triple backtick spans, an escaped backtick, and a one-line triple-backtick span at the start of a line.
- `crates/mindtape/src/body/content_gate/content_gate_density.rs`: mask each inline raw span byte for byte, so splits and counts run on the masked text and the snippet comes from the original; treat a line whose opening backtick run closes on that line as inline, not a fence.
- `www/content/rules.typ`: the density bullet says code, fenced or inline, is removed first.


== Alternatives

- Remove the spans outright. A line holding only a span then turns blank and splits a paragraph.
- A `skip-code` key on the assertion. Nobody wants a density count over code, so a key would only add a way to get it wrong.
- Strip spans in the rule's regex. A regex can't tell a backtick that opens a span from one that closes it.


== Out of scope

- `must-match` and `must-not-match`, which read the raw section on purpose, as `show-shape-when-gating` looks for a fence.
- Code written as `#raw(...)` calls, which no task here uses.


== Dogfood

The `no-open-questions` rule this repo plans counts `?` per sentence, and needs the code spans in its TO-BE bullets skipped.

