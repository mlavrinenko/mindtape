#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "mt exits 141 on a broken pipe about one call in ten",
  priority: ice(
    impact: 8,
    confidence: 0.95,
    ease: 5,
  ),
  difficulty: 3,
  tags: (),
  estimate: (files: 3),
  assignee: "max",
  status: done(
    2026,
    8,
    29,
  )[not mt: the 141 came from the nix mkCargoShim wrapper on PATH, whose staleness check ran `find | sort -nr | head -1` under `set -euo pipefail`. `head` closed the pipe, `sort` died of `SIGPIPE`, and the wrapper exited 141 before executing mt. Fixed at source (`sort -n | tail -1`): 13/200 before, 0/300 after. mt hardened anyway, so a closed reader is an ordinary write error instead of a panic at exit 101. `tests/broken_pipe_e2e.rs` pins never-141, never-signalled.],
)

== Summary

`mt` 0.3.0 exits 141 (`SIGPIPE`) on a substantial fraction of calls, with no
output and no error. Measured twice on the `kimmable` board (57 tasks): 4/200 and
8/300 by one observer, then 15/150, 10%, by a second on the same machine. It is
not the caller's pipe: the same rate appears with stdout redirected to a regular
file. The signal originates inside `mt` or a process it spawns.

The failure is silent and easy to miss. Piping `mt ls` into `rg` returns an empty
list rather than an error. A caller then reads "no tasks" and believes it. That
happened live while writing this: a board listing came back empty and was only
caught because the emptiness was implausible.

It is worst for anything filing in a loop. The `kimmable`
`just ask-refactoring` makes ~25 `mt` calls per sweep. Three consecutive runs
died at finding 1, finding 13 and finding 1. Each left a half-filled board that
the sweep's own re-run guard then refused to let anyone finish. The signal was
perfectly good, and the backlog stayed wedged. It carries
`contrib/ask-refactoring/mt.sh`, which retries on 141 and nothing else. That
workaround should not need to exist.


== Scope

- Find where the signal starts. Suspect a stdout writer that loses its reader
  while a child or a rayon/tokio worker still holds a handle, or a `println!`
  racing shutdown. The file-redirect result rules out the obvious pipe reader.
- Decide the policy deliberately. Either install a `SIGPIPE` disposition at
  startup and let a broken pipe surface as an ordinary write error, or find and
  fix the premature close. Rust sets `SIGPIPE` to ignore at startup, and `libstd`
  turns the write into `EPIPE`. An exit of 141 means something restored the
  default handler, or the signal reached a thread that had not.
- A regression test runs `mt ls` hundreds of times and asserts no exit is 141.
  It must fail first: confirm it goes red against today's binary before it goes
  green.
- Once fixed, tell `kimmable` so `contrib/ask-refactoring/mt.sh` can lose the
  retry loop.
