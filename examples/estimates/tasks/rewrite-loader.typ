#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rewrite the config loader",
  status: todo(2026, 9, 1),
  priority: 7,
  difficulty: 3,
  // Judgment work, so the gated ceiling applies: 5 files, not 15. A sixth
  // file here fails `mt check` with the rule's own message.
  estimate: (files: 5),
)

= Summary

Fold the four layers into one resolve pass so an origin is reported per key.
