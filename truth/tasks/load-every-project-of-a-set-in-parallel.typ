#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Load every project of a set in parallel",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[30 projects read straight from their files stay fast with no index],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
  links: parent("show-every-tracked-project-from-one-mt-name.typ"),
)

== Problem

+ Run `time (for d in ~/projects/home/*/.mindtape; do mt -w "${d%/.mindtape}" ls '!terminal' --format path >/dev/null; done)` over 30 projects with a warm cache.
+ 🔴 It takes 2.9s, while the same 30 runs started at once take 0.11s: loading one project after another leaves the cores idle.


== TO-BE

- 🟢 `mindtape::load_many` loads each root's config and tasks on its own scoped thread and returns them in the order given. A broken config is that member's error, not the run's.
- 🟢 `util::open_set` warns per member, naming the project in each config warning and load error.
- 🟢 A held-back command is recorded with its project, and the warning names `mt @<project> allow`.
- 🟢 30 warm projects load in about the time of the slowest one.


== Changes

- `crates/mindtape/src/loader/many_tests.rs`: the red tests first: results keep the input order, and a broken config fails only its member.
- `crates/mindtape/src/loader/many.rs`: `load_many` over `std::thread::scope`.
- `crates/mindtape/src/loader/mod.rs`: declare it in the `//!` map.
- `crates/mindtape/src/allow.rs`: key held-back commands by project root.
- `crates/mindtape/src/lib.rs`: re-export `load_many`.
- `crates/mindtape-cli/src/util.rs`: `open_set` over `(name, root)` pairs, and warnings that name their project.


== Alternatives

- `rayon` in `mindtape`: `typst-harvest` already runs a project's files on rayon, and a scoped thread per project needs no new dependency.
- A shared index of every project: the setup the `plain-files` goal rules out.
- One project after another: 2.9s for 30 projects.


== Out of scope

- Using the loaded set: `ls`, `check` and `board` do in their own tasks.
- A cap on threads: a set of 30 is 30 threads, and rayon bounds the harvest under them.


== Dogfood

The owner's `mt @home` loads this repo beside the 29 other projects under `~/projects/home`.

