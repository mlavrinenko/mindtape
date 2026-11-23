// docs:slice ship-the-search-index
#import "@local/mindtape:0.5.0": *
#import "/goal-calls.typ": *

#show: task.with(
  title: "Ship the search index",
  status: proposed(2026, 6, 29),
)

= Summary

Search scans every note on every keystroke, and a large folder pauses after
the first letter.

== Goals

+ #Helps(to: answer-fast)[results appear while the query is still being typed].
+ #Neutral(avoid-lock-in).
// docs:end
