#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Skip the config schema walk when no pack parameter is used",
  status: done(2026, 11, 27),
  tags: ("tweak",),
)

A config with no `param()` built the whole `Config` JSON Schema at runtime to find the keys that hold expressions, then walked it to find nothing to rewrite.
