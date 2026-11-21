set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list session

# The session start. `mt config show` goes through the essentials report, so the
# roots and statuses are the project's own. One listing pass reads the task
# files, over every task because that single read carries both counts. The
# templates and the allow sites are one line each.

[doc("Show the project's shape, the open count, the templates and the allow status")]
[group("session")]
[script]
orient:
    essentials=$(mt config show --json --resolved | jq -f contrib/jq/reports/essentials.jq)
    jq -r '
      "project:",
      "  roots: \(.tasks.roots | map(.path + (if .tag then " (tag: " + .tag + ")" else "" end)) | join(", "))",
      "  statuses:",
      "    open: \(.tasks.statuses | map(select(.terminal | not) | .value) | join(", "))",
      "    terminal: \(.tasks.statuses | map(select(.terminal) | .value) | join(", "))"
    ' <<<"$essentials"
    terminal=$(jq -c '[.tasks.statuses[] | select(.terminal) | .value]' <<<"$essentials")
    mt ls --json '' | jq -r --argjson terminal "$terminal" '
      "open: \([.tasks[] | select(.status.name as $s | $terminal | index($s) | not)] | length) of \(.tasks | length) tasks"
    '
    mt templates --json | jq -r '
      "templates: \([.templates[].name] | join(", ")) (default: \([.templates[] | select(.default) | .name] | first // "none"))"
    '
    mt allow --status --json | jq -r '
      . as $a
      | "allow: "
        + (if ($a.sites | length) == 0 then "nothing to allow"
           else ([$a.sites[].command | if type == "array" then join(" ") else . end] | join(", "))
             + (if $a.allowed then "" else " (not allowed yet)" end)
           end)
    '
