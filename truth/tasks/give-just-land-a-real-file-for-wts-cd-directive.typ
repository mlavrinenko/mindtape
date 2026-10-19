#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Give just land a real file for wt's cd directive",
  status: done(2026, 10, 20),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The recipe pointed WORKTRUNK_DIRECTIVE_CD_FILE at /dev/null and wt ended with a shell-integration warning
