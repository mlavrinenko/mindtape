#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Drop the finding template's redundant carrier line",
  status: done(2026, 11, 4),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The `carrier` slot repeats the first `== Changes` bullet, so the two can drift while `Changes` already names every fix file.
