#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Parse CSV rows",
  status: done(2026, 6, 28),
  priority: 7,
  links: parent("ship-the-csv-importer.typ"),
)

= Summary

Read quoted fields and embedded newlines the way RFC 4180 describes them.
