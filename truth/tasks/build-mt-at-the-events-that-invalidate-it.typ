#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build mt at the events that invalidate it",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[CONTRIBUTING.md names when mt is built instead of claiming a rebuild before every call],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 28),
  links: related(
    "rebuild-mt-before-every-run-in-the-dev-shell.typ",
  )[supersedes its per-call rebuild wrapper with build-at-the-events],
)

== Problem

+ Enter the dev shell and run `mt --version`.
+ 🔴 A no-op run costs ~167 ms against ~6 ms for `target/mt-dev/debug/mt` invoked directly: the wrapper runs `cargo build -q` before every call.
+ Run `mt` again with no edit between: the same ~167 ms, because the build fires per call rather than at the events that invalidate the binary.


== Environment

- NixOS, flake dev shell via direnv; `mt` 0.3.0 through the `mt-dev` wrapper added by commit `be437b71`.
- The wrapper exists because a bare `target/debug` on `PATH` once answered from a stale or absent binary (`truth/tasks/rebuild-mt-before-every-run-in-the-dev-shell.typ`).


== TO-BE

- 🟢 The dev shell prepends `<root>/target/mt-dev/debug` to `PATH`, so `mt` is a plain binary and no build rides a call.
- 🟢 `just build-mt` builds it; `just fork` builds it once for the new worktree, `just integrate` rebuilds this checkout's after the fast-forward. An edit between those events leaves `mt` stale until the next build — the accepted trade.
- 🟢 Entering the shell with no binary there prints one line naming `just build-mt` and what `mt` currently resolves to, only when stderr is a terminal.


== Changes

- `flake.nix`: drop the `mt-dev` wrapper and `MINDTAPE_DEV_ROOT`; the shellHook prepends the dev target dir to `PATH` and prints the missing-binary hint under `[ -t 2 ]`.
- `Justfile`: a `build-mt` recipe (`cargo build -q -p mindtape-cli --bin mt --target-dir target/mt-dev`), the one command every other builder calls.
- `.just/scripts/fork.sh`: after the dev shell loads, build the new worktree's binary so the delegated agent's first `mt` works.
- `.just/scripts/integrate.sh`: rebuild this checkout's binary after the fast-forward, before the worktree is removed.
- `.github/workflows/ci.yml`: the commit-rules step runs `cargo run -q -- check --commits …`, the pre-commit hook's spelling, since no `mt` is on PATH outside a dev shell.
- `docs/src/contributing.typ`, regenerated `CONTRIBUTING.md`, re-recorded `outdatty.lock`: the quick-start line names the events that build `mt`, and the fork/integrate paragraph loses "no warmed cache" for the one build fork now runs.


== Out of scope

- A file watcher or a rebuild-on-change hook: the build happens at the events that invalidate the binary, and `just build-mt` covers the rest.
- Building inside the shellHook on every load: `nix develop --command` runs it, so CI would pay a sequential compile.
- `target/debug/mt`: the rrule arm builds that path with a feature this default build would relink away.


== Dogfood

Every `mt` call typed in this repo's dev shells stops paying the build; delegated worktrees get their binary once at `just fork`.

