#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Point outdatty at test_support's split files",
  status: done(2026, 10, 14),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

`outdatty.yaml` named `crates/mindtape/src/test_support.rs`, gone since the split into a folder, and every outdatty run warned about it.
