// examples/rules/lib.typ — the function the `acceptance-criteria` rule calls.
// It receives the task's `x-acceptance` value, or `none` when the task has
// none, and returns one string per problem. An empty array passes.
#let check-acceptance(criteria) = {
  if criteria == none {
    return ("list what must be true to close this task in x-acceptance",)
  }
  if type(criteria) != array or criteria.len() == 0 {
    return ("x-acceptance is a non-empty list of criteria",)
  }
  criteria
    .enumerate()
    .filter(((i, c)) => type(c) != str or c.trim() == "")
    .map(((i, c)) => "acceptance criterion " + str(i + 1) + " is empty")
}
