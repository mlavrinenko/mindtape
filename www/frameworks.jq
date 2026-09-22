# www/frameworks.jq — the jq program www/generate-facts.sh runs over
# crates/mindtape/src/frameworks.toml (piped in as JSON by tomlq) to build
# frameworks.json, with the two scales those frameworks score onto passed in
# as $scales from crates/mindtape/src/defaults.toml. A sibling of
# www/gates.jq and www/sweep-flags.jq, split out for the same two reasons:
# a derivation with an argument to write does not belong in a shell string,
# and www/generate-facts.sh has no linecop budget to spend on one.
#
# Subject: the baked-in priority frameworks and the scales they land on
# (truth/tasks/derive-the-built-in-priority-tables.typ). /priority/ used to
# hand-type the five names, their input lists, and the shipped difficulty
# anchors, so adding, renaming or re-ranging any of them shipped a page that
# still described the old one. Everything MECHANICAL is derived here — names,
# input names and types and bounds, the score formula, the category labels,
# the bands and anchors. What a formula MEANS stays prose, in
# www/utils/framework-notes.typ, because no source declares it.
#
# Key order is the TOML's own: frameworks in declaration order (ice, rice,
# wsjf, eisenhower, moscow — the order the file argues them in, coarse to
# categorical) and each framework's inputs in the order its call takes them,
# so the rendered table reads as `ice(impact:, confidence:, ease:)` does.
# Sorting either would make the page disagree with the DSL call it documents.
#
# Absent keys are emitted as null rather than omitted, so the Typst renderer
# branches on a value instead of on a key's existence: `min`/`max` are null
# for a bool input, `values` is null for anything but an enum, and `score` is
# null for a categorical framework. That last null is the fact the page reads
# to say which frameworks band a number and which classify into categories.

# Every framework must declare inputs and exactly one scoring shape — a
# scalar `score` formula or a `category` list — which is the contract
# frameworks.toml's own header states. Checked here rather than trusted
# because the renderer cannot see the difference: a framework with neither
# shape renders a blank Score cell and a framework with both renders the
# formula while `mt` uses the categories, and both read as a finished page.
def check_shape:
  [ to_entries[]
    | select((.value.inputs // {} | length) == 0)
    | "  " + .key + ": no inputs"
  ]
  + [ to_entries[]
      | select((.value.score != null) == (.value.category != null))
      | "  " + .key + ": needs exactly one of `score` or `category`"
    ]
  | if length > 0 then
      # Trailing newline for www/gates.jq's reason: `halt_error` writes its
      # string verbatim, so without one the message runs into the shell prompt.
      ("frameworks.jq: malformed framework declarations:\n" + join("\n") + "\n")
      | halt_error(1)
    else empty end;

# The scales are a separate file's tables, so a rename there reaches this
# program as a null rather than as a missing input, and would emit a fact
# whose ranges are silently empty. Fail by name instead.
def check_scales:
  [ "priority", "difficulty" ]
  | map(select($scales[.] == null))
  | if length > 0 then
      ("frameworks.jq: no [tasks." + (join(", tasks.")) + "] in defaults.toml\n")
      | halt_error(1)
    else empty end;

def input_rows:
  [ (.inputs // {}) | to_entries[]
    | { name: .key,
        type: .value.type,
        min: .value.min,
        max: .value.max,
        values: .value.values }
  ];

def category_rows:
  [ (.category // [])[]
    | { label, when, score, glyph, color }
  ];

check_shape,
check_scales,
{ frameworks: [ to_entries[]
                | { name: .key,
                    score: .value.score,
                    inputs: (.value | input_rows),
                    categories: (.value | category_rows) } ],
  scales: { priority: $scales.priority,
            difficulty: $scales.difficulty } }
