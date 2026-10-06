# www/status-moves.jq — turn a project's `mt config show --json` into the DOT
# graph of its status legend, which www/capture.sh's render_dot draws for
# /statuses/.
#
# Each status is a node and each name in its `to` list is an edge. A terminal
# status gets a double border, so a reader tells where work ends from where it
# pauses. A legend in which no entry declares `to` allows every move, and a
# graph of that is a complete graph that says nothing, so the program fails on
# one rather than draw an empty graph.
.tasks.status as $legend
| if ($legend | length) == 0 or all($legend[]; has("to") | not) then
    error("status-moves.jq: the legend declares no `to`, so it has no moves to draw")
  else . end
| [
    "digraph legend {",
    "  rankdir=LR;",
    "  node [shape=box, style=rounded, fontname=\"sans-serif\"];",
    ($legend[]
      | "  \(.value | @json) [label=\((.glyph // "" | if . == "" then "" else . + " " end) + .value | @json)"
        + (if .terminal then ", peripheries=2" else "" end) + "];"),
    ($legend[] | .value as $from | (.to // [])[]
      | "  \($from | @json) -> \(. | @json);"),
    "}"
  ]
| .[]
