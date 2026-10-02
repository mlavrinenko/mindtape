#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add CSV export to the report page",
  status: todo(2026, 7, 1),
  kind: "feat",
  priority: 5,
  assignee: "lin",
  x-acceptance: ("the export matches the table on screen, column for column",),
)

= Summary

Finance copies the report table into a spreadsheet by hand every month.

= Notes

The table already sorts on the server, so the export can reuse that query.
