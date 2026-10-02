#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Map CSV columns to task fields",
  status: cancelled(2026, 6, 29)[the importer reads the header row as-is],
  priority: 4,
  links: parent("ship-the-csv-importer.typ"),
)

= Summary

Let the user pick which column becomes the title.
