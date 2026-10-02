// docs:slice ship-onboarding-checklist
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Ship the onboarding checklist",
  status: done(2026, 7, 1),
  priority: 5,
  x-goals: (
    fast-onboarding: "cuts time-to-first-task from ten minutes to two",
    no-lock-in: "reads and writes the same plain files a human would edit",
  ),
)
// docs:end

= Summary

A checklist screen new users see once, on their first login, that walks them
to their first completed task instead of an empty dashboard.
