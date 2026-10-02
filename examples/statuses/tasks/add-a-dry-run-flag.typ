#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add a dry-run flag to the importer",
  status: wip(2026, 7, 1),
  priority: 6,
  links: depends-on("parse-csv-rows.typ"),
)

= Summary

Print the tasks an import would write, and write nothing.
