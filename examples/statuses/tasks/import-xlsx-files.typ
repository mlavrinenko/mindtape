#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Import XLSX files",
  status: proposed(2026, 7, 2),
  priority: 5,
  links: depends-on("add-a-dry-run-flag.typ"),
)

= Summary

Accept a spreadsheet wherever a CSV file is accepted.
