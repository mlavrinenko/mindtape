// What MindTape is for. Every open task gives each goal a verdict through
// `goals(product: ...)` from truth/lib.typ. How this repo builds MindTape is
// a separate set, in process-goals.typ.

#let goals = (
  readable-tasks: (
    title: "Tasks read at a glance",
    acid-test: [Anyone opening an open task file can say within 30 seconds what
      is broken, why it matters, and how it should work instead.],
  ),
  implementable-tasks: (
    title: "Tasks guide the implementer",
    acid-test: [An agent in a fresh session, given only the task file, lands the
      change without asking where to start or what done looks like.],
  ),
  enforced-practice: (
    title: "Checks run where work happens",
    acid-test: [`mt add` refuses a task that breaks the project's rules, and
      `mt flip` runs the project's own checks before a status move lands; no
      rule depends on someone remembering it.],
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
)
