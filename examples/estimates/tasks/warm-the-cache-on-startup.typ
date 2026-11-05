#import "@local/mindtape:0.4.0": *

#show: task.with(
  title: "Warm the cache on startup",
  status: todo(2026, 9, 3),
  priority: 4,
  difficulty: 3,
  // docs:slice shape
  // One new call site, plus every file that already calls it.
  estimate: (files: count(1, ast_grep("rust", `warm($$$ARGS)`))),
  // docs:end
)

= Summary

Warm the harvest cache before the first request, so the first read pays for it.
