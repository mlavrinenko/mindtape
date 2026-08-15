# www/sweep-flags.jq — the jq program www/generate-facts.sh runs over `just
# --dump --dump-format json` (as $JUST_DUMP, piped in) to build
# sweep-flags.json. A sibling of www/gates.jq, split out of the shell script
# for that program's reason and one more: www/generate-facts.sh is nine lines
# under its own linecop cap, so a derivation with an argument to write cannot
# live inline there at all.
#
# Subject: `just ask-refactoring`'s flag interface — the signal sweep's five
# thresholds (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-
# attributes.typ). Every one of them was prose in a comment above the recipe
# until they became declared parameters carrying `[arg(...)]` attributes.
# `just --dump` reports each parameter with its `long`, `short`, `help`,
# `pattern` and (for a flag that sets a fixed value rather than taking one)
# `value`, and carries the DEFAULT alongside in the same object — so the
# defaults column of docs/contributing/signal-sweep.md's table is read out of
# the Justfile rather than retyped beside it, which is the whole point.
#
# Row order is the recipe SIGNATURE's order, not sorted: that is the order
# `just --usage ask-refactoring` prints and the order the flags read in
# (`--write`, then one arm at a time). Sorting here would make the rendered
# table disagree with just's own output for no gain.
#
# The recipe name is hardcoded, like `.recipes.check` is in gates.jq. There is
# exactly one swept recipe and inventing a discovery rule for a set of one
# would be a guess about a second one that does not exist.

def recipe_name: "ask-refactoring";

.recipes[recipe_name] as $r
| if $r == null then
    ("sweep-flags.jq: no `" + recipe_name + "` recipe in the just dump")
    | halt_error(1)
  else . end
# A parameter with no `long` is positional-only, so nothing renders it as a
# flag and a doc table row for it would name no flag at all. That is the
# regression this check exists for: dropping the `[arg(...)]` attribute while
# keeping the parameter leaves the sweep working and the docs quietly wrong.
| ([$r.parameters[] | select(.long == null) | .name]) as $flagless
| if ($flagless | length) > 0 then
    ("sweep-flags.jq: " + recipe_name + " parameters with no [arg(long=...)]: "
      + ($flagless | join(", ")))
    | halt_error(1)
  else . end
| {
    recipe: recipe_name,
    flags: [$r.parameters[]
      | {
          name,
          long,
          short,
          help,
          default,
          pattern,
          # Set when the flag takes no argument of its own and assigns a fixed
          # value instead (`--write` sets 1); null when it takes one.
          value,
        }],
  }
