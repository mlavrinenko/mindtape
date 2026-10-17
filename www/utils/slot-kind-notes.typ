// What each template slot kind holds, keyed by the kind's spelling. The
// /templates/ page reads the kind list from www/generated/slot-kinds.json,
// which www/generate-facts.sh reads off `SlotKind::ALL` in
// crates/mindtape/src/template/mod.rs, so this dictionary holds only the
// meaning. `just check-doc-facts` fails on a kind in that list with no entry
// here, and the page fails to build on the same gap.

// Each note is a phrase without a final period: the page joins them into one
// sentence as "`kind` is <note>".

#let slot-kind-notes = (
  "string": [a quoted string, and the default kind],
  "int": [a whole number],
  "float": [a decimal number],
  "real": [a decimal number, kept apart from `float` so a `real` scale reads back as `real`],
  "bool": [`true` or `false`],
  "date": [a `YYYY-MM-DD` date],
  "list": [a list of strings],
  "text": [body markup that `mt add` escapes, so `#`, `*` and `_` print as typed],
  "raw": [body markup pasted in as written, which `mt add` refuses if it doesn't parse],
  "choice": [one of the slot's `choices`],
  "multichoice": [a list whose items are each one of the slot's `choices`],
)
