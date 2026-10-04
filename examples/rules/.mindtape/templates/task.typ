#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot(
    "title",
    prompt: "What changes, with no full stop at the end",
    example: "Add a CSV export to the report page",
  ),
  kind: slot("kind", prompt: "feat or bug", example: "feat"),
  priority: slot("priority", kind: "int", prompt: "0-9", example: 5),
  assignee: slot("assignee", prompt: "ada or lin", example: "lin"),
  estimate: slot(
    "estimate",
    kind: "raw",
    required: false,
    prompt: "(hours: N), which a bug needs",
    example: "(hours: 3)",
  ),
  x-acceptance: slot(
    "acceptance",
    kind: "list",
    prompt: "What must be true to close the task",
    example: ("the export matches the table on screen, column for column",),
  ),
)

= Summary

#slot(
  "summary",
  kind: "raw",
  prompt: "Why the task exists, in a paragraph",
  example: ```typ
  Finance copies the report table into a spreadsheet by hand every month.
  ```,
)
