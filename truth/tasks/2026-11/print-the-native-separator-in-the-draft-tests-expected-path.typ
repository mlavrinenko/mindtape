#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Print the native separator in the draft test's expected path",
  status: done(2026, 11, 12),
  tags: ("tweak",),
)

`Path::display` keeps the `/` a relative literal holds, so the expectation stayed POSIX on Windows
