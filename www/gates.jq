# www/gates.jq — the jq program www/generate-facts.sh runs over `just --dump
# --dump-format json` (as $JUST_DUMP, piped in) to build gates.json. Split out
# of the shell script because it is long enough on its own to blow that
# script's linecop cap; kept as `jq -f`, the same external-program pattern
# `contrib/jq/*.jq` already uses for the import cookbook.
#
# Gate membership + row order both come from the Justfile: every gate recipe
# carries `[group("gate")]` (truth/tasks/tag-just-recipes-with-groups.typ).
# The JSON dump sorts recipes alphabetically and carries no source order, so
# row order is read back from `check`'s own dependency list instead — see the
# "row order" paragraph below for why that changed from the prior derivation.
#
# Cross-checked against that same (mmz-unwrapped) dependency list for
# membership — the old derivation got membership-equals-dependencies for free
# by construction, and losing that silently would let a recipe carry
# `[group("gate")]` without actually gating anything, or gate something with
# no `[group("gate")]` tag to show for it. Disagreement fails here, naming
# both sides of the mismatch, rather than downstream in check-doc-facts.
#
# A gate can live in a module now (truth/tasks/split-the-justfile-into-
# modules.typ), so recipe collection walks `.modules` recursively (all_recipes
# below) instead of reading `.recipes` alone — a module recipe not walked here
# would silently drop out of the gate table the day it moved. The two sets
# compared, $gate_group and $check_deps, use the MANGLED identifier (`::`
# joined with `-`, same function www/generate-facts.sh's just.typ producer
# uses). `check`'s dependency list carries the raw `::` form (it is a literal
# invocation string, e.g. `docs::check` passed to `mmz`), so it is mangled
# here too before comparing. The gate's own `name` field is the recipe's real
# invocation path, `docs::check`: docs/src/contributing.typ prints it verbatim
# (`raw(g.name)`) and www/utils/gate-notes.typ's dict is keyed by it.
#
# `check` runs its arms in stages, each a private recipe (`_check-fast`, …)
# whose dependencies are the mmz arms, so `arms` below reads through a private
# dependency to its own list. Row order is then stage order, and within a
# stage the order the stage lists its arms.
#
# Row order: this used to read `just --summary --unsorted` (source order,
# passed in as $order by www/generate-facts.sh) — truth/tasks/tag-just-
# recipes-with-groups.typ, "make the Justfile the single source of both
# membership and order". A module recipe breaks that: verified on just 1.46,
# `--summary --unsorted` always lists EVERY top-level recipe first and appends
# module recipes after, in each module's own declaration order, regardless of
# where the `mod` statement sits among the top-level recipes. docs::check and
# docs::md-check would have jumped from between check-doc-recipes and
# fmt-fixture-check (their pre-split neighbors) to the very end of the table —
# a rendered CONTRIBUTING.md diff with no doc source touched. `check`'s own
# dependency list does not have this problem: it is one literal argument list
# the split already had to touch by hand (mmz's string argument, renamed in
# place, never reordered), so its textual order survives a recipe moving
# module exactly as well as it survives one changing group. It was already
# required to equal $gate_group as a set; reading row order from the same
# list makes the `check` line the one place that changes reorder the table,
# instead of two places (that line for membership, physical recipe position
# for order) that discipline alone kept in sync.
#
# `assignments` folds in the Justfile's fmt path variables
# (TOML_FMT_PATHS/TYPST_FMT_PATHS/JUST_FMT_PATHS) verbatim, so
# docs/src/contributing/gates.typ can render their real contents instead of
# sending the reader to the Justfile — reusing $JUST_DUMP, never a second
# `just --dump`. The selection is the `_FMT_PATHS` suffix the Justfile's own
# grouping discipline gives every fmt list, not a name list, so a fifth
# formatter's list (a shell formatter, say) lands in the artifact without an
# edit here; CRAP_EXCLUDES is deliberately not carried — it is an exclude
# list, not a fmt path list, and no doc renders it.

# Every recipe object, top-level and module, flattened. Each recipe already
# carries its own fully-qualified `namepath` (just's own dump computes it,
# e.g. a `docs` module's `check` recipe reports "docs::check"), so no prefix
# bookkeeping is needed here — just walk `.modules` recursively and collect.
def all_recipes:
  ((.recipes // {}) | to_entries | map(.value))
  + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);

def mangle: gsub("::"; "-");

(all_recipes) as $recipes
| ($recipes | map({key: (.namepath | mangle), value: .}) | from_entries) as $by_mangled
| ($recipes
    | map(select(.attributes // [] | any(.group? == "gate")))
    | map(.namepath | mangle) | sort) as $gate_group
| ($recipes | map({key: .namepath, value: .}) | from_entries) as $by_path
| ([def arms: .dependencies[]
      | if .recipe == "mmz" then .arguments[0]
        elif $by_path[.recipe].private then $by_path[.recipe] | arms
        else .recipe end;
    .recipes.check | arms | mangle]) as $check_deps_ordered
| ($check_deps_ordered | sort) as $check_deps
| if $gate_group != $check_deps then
    # `(none)` rather than an empty join, and a trailing newline rather than
    # none, because BOTH omissions were read off one real failure (truth/tasks/
    # fail-a-half-wired-gate-arm-at-the-wiring-it-is-missing.typ): a recipe
    # carrying [group("gate")] but absent from `check` empties the second side,
    # and `halt_error` writes its string verbatim — no newline appended — so
    # www/generate-facts.sh's own next stderr line landed after that empty
    # colon and read as a check dependency named `generate-facts.sh: refusing
    # to install malformed gates.json`. The one line a reader would act on was
    # the fiction. An empty side must render as a value, and the message must
    # end where it ends.
    (def side: if length == 0 then "(none)" else join(", ") end;
      "gate group and check dependencies disagree:\n"
      + "  only in [group(\"gate\")]: " + (($gate_group - $check_deps) | side) + "\n"
      + "  only in check dependencies: " + (($check_deps - $gate_group) | side) + "\n")
    | halt_error(1)
  else
    ($check_deps_ordered) as $names
    | {
        gates: [$names[] as $n
          | $by_mangled[$n] as $r
          | {name: $r.namepath}],
        assignments: (.assignments
          | with_entries(select(.key | test("_FMT_PATHS$")))
          | map_values(.value)),
      }
  end
