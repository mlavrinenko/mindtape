#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Pick an auth provider",
  status: done(2026, 8, 4)[compared three providers, chose the self-hosted one],
  priority: 6,
  tags: ("auth",),
)

= Summary

Compare hosted auth providers against the self-hosted option, then record the
decision in an ADR.
