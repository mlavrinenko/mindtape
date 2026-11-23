#import "@local/mindtape:0.5.0": *
#import "/goal-calls.typ": *

#show: task.with(
  title: "Pin the search schema",
  status: proposed(2026, 6, 30),
)

= Summary

The search index format changes with every release, so a downgrade loses the
index and forces a full rebuild.

== Goals

+ #Neutral(answer-fast).
+ #Helps(to: avoid-lock-in)[the index keeps a documented format an older release reads].
