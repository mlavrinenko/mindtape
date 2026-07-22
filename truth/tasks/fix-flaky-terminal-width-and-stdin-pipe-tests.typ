#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "fix flaky terminal-width and stdin-pipe tests",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  tags: (),
  status: done(2026, 7, 21),
)

== Summary

Two CLI tests read global state and raced: apply_flex queried the real terminal so the None-guard test failed under a TTY, and the stdin-title inner-child test wrote into a pipe the early-returning child never reads, panicking on BrokenPipe under tarpaulin's slower timing. Inject terminal width into apply_flex and tolerate BrokenPipe on the unread-pipe write.

== Scope

- TODO
