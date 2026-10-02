// docs:slice export-report-as-csv
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Export the report as CSV",
  status: wip(2026, 7, 1),
  priority: 4,
  x-goals: (
    fast-onboarding: "not directly, but keeps the report screen simple",
  ),
)
// docs:end

= Summary

Add a CSV export button beside the existing PDF one, using the report's
already-computed rows.
