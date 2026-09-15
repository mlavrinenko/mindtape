// MindTape's goals. Every open task imports this file and says, in its Goals
// section, how it affects each goal. `goals(...)` refuses a task that skips a
// goal, names an unknown one, gives helps or hurts no reason, or helps none:
// a task nothing justifies is cancelled, not filed.

#let registry = (
  readable-tasks: (
    title: "Tasks read at a glance",
    acid-test: [Anyone opening an open task file can say within 30 seconds what
      is broken, why it matters, and what done looks like.],
  ),
  implementable-tasks: (
    title: "Tasks guide the implementer",
    acid-test: [An agent in a fresh session, given only the task file, lands the
      change and proves it with the task's own steps, without asking.],
  ),
  enforced-practice: (
    title: "The tool enforces practice",
    acid-test: [A task or config that breaks the project's practice cannot be
      written by `mt` or committed; no rule depends on someone remembering it.],
  ),
  cheap-adoption: (
    title: "Best practice adopts in one line",
    acid-test: [A repo takes the flagship's practice with one config line,
      overrides what it disagrees with, and gets later fixes without copying.],
  ),
  typst-over-markdown: (
    title: "Typst beats Markdown with frontmatter",
    acid-test: [A software engineer reading the docs' first screen sees a task
      that Markdown plus YAML plus scripts cannot express as briefly.],
  ),
  plain-files: (
    title: "Plain files, no infrastructure",
    acid-test: [`git clone` plus the `mt` binary is the whole setup: no daemon,
      no database, no index, and nothing outside the repo to lose.],
  ),
  sound-codebase: (
    title: "Changes stay safe",
    acid-test: [A change that breaks documented behaviour fails `just check`
      before it can be committed, and every file fits its size cap.],
  ),
)

#let helps(reason) = (effect: "helps", reason: reason)
#let hurts(reason) = (effect: "hurts", reason: reason)
#let neutral = (effect: "neutral", reason: none)

#let _markers = (helps: "👍", hurts: "👎", neutral: "➖")
#let _choices = "helps[...], hurts[...] or neutral"

#let goals(..impact) = {
  if impact.pos().len() > 0 {
    panic("goals(...) takes one named argument per goal")
  }
  let given = impact.named()
  for key in registry.keys() {
    if key not in given {
      panic("goal `" + key + "` is not addressed; say " + _choices)
    }
  }
  for (key, value) in given {
    if key not in registry {
      panic(
        "unknown goal `"
          + key
          + "`; the goals are "
          + registry.keys().join(", "),
      )
    }
    if type(value) != dictionary or "effect" not in value {
      panic("goal `" + key + "` takes " + _choices)
    }
    if value.effect != "neutral" and value.reason in (none, [], [ ]) {
      panic("goal `" + key + "` says " + value.effect + " but not how")
    }
  }
  if given.values().all(value => value.effect != "helps") {
    panic("the task helps no goal; if nothing justifies it, cancel it")
  }
  list(..given
    .pairs()
    .map(((key, value)) => [
      #_markers.at(value.effect) *#registry.at(key).title*#if value.reason != none [: #value.reason]
    ]))
}
