# contrib/jq/reports/essentials.jq: the project's shape, read off the resolved
# config. It answers what this project is without loading a task.
#
#   mt config show --json --resolved | jq -f contrib/jq/reports/essentials.jq
#
# The output's top-level keys are the config's own, less `dropped`. Every key
# this report leaves out is named there, so a reader can tell a section that is
# empty in this project from one the report forgets to print.
#
# `jq -n -f contrib/jq/reports/essentials.jq` prints `dropped` alone. The gate
# reads the array that way, without a live config on stdin.
#
# TEMPORARY: the `moments` on each rule are this repo's own reading of the four
# commands, kept here until the mapping settles. The report retires when
# `git log --since="1 month ago" -- contrib/jq/reports/` is empty at a release
# review, which means no one has had to correct it for a month.

# The top-level config keys this report deliberately leaves out.
# `disable`, `extends` and `pack-sources` say how the layers combine rather
# than what the config does, and `--resolved` omits them too. `format`, `ls`
# and `typst` describe the writer, the listing and the compiler, not the
# project's shape.
def dropped: ["disable", "extends", "format", "ls", "pack-sources", "typst"];

# A rule with the command or commands that check it, and a label for the
# unnamed case. Body rules and `[[tasks.rules]]` reach `mt add` and `mt flip`;
# a commit rule reaches `mt check --commit-msg` and `--commits`; a hook reaches
# its own flip or rm.
def labelled($section; $moments; $rules):
  ($rules // [])
  | to_entries
  | map(
      .value as $rule
      | $rule
      + {
          section: $section,
          label: ($rule.name // "\($section)[\(.key)]"),
          moments: $moments
        }
    );

# Every `x-` field the config declares or a `check` rule reads, with the rules
# that read it. A declared field with no reader stays, so an extension is
# visible before the rule that enforces it arrives.
def extensions($tasks):
  (
    [
      (($tasks.fields // {}) | keys[])
      | select(startswith("x-"))
      | { name: ., rules: [] }
    ]
    + [
        ($tasks.rules // [])[]
        | select((.check.field // "") | startswith("x-"))
        | { name: .check.field, rules: [(.name // "unnamed")] }
      ]
  )
  | group_by(.name)
  | map({ name: .[0].name, rules: ([.[].rules[]] | unique) })
  | sort_by(.name);

# The priority scale, the framework that scores it, and each input's own
# bounds. A project that scores priority by hand keeps the scale and no
# framework, so the framework keys appear only when one is set.
def priority($p):
  (if $p.framework == null then null else $p.frameworks[$p.framework] end) as $framework
  | {
      framework: $p.framework,
      kind: $p.kind,
      min: $p.min,
      max: $p.max,
      required: $p.required,
      "allow-manual": $p."allow-manual",
      score: $framework.score,
      categories: $framework.category,
      inputs: $framework.inputs
    }
  | with_entries(select(.value != null and .value != []));

def report($c):
  {
    actors: $c.actors,
    add: $c.add,
    commits: {
      trailer: $c.commits.trailer,
      run: $c.commits.run,
      "external-refs": $c.commits."external-refs",
      rules: labelled("commits.rules"; ["commit"]; $c.commits.rules)
    },
    dicts: ($c.dicts | with_entries(.value |= { case, include, values })),
    go: $c.go,
    on: {
      add: labelled("on.add"; ["add"]; $c.on.add),
      flip: labelled("on.flip"; ["flip"]; $c.on.flip),
      rm: labelled("on.rm"; ["rm"]; $c.on.rm)
    },
    project: $c.project,
    retire: $c.retire,
    rm: $c.rm,
    roles: $c.roles,
    tasks: {
      roots: [$c.tasks.roots[] | { path, tag: .name, exclude, "derive-tags" }],
      statuses: $c.tasks.status,
      assignee: $c.tasks.assignee,
      priority: priority($c.tasks.priority),
      difficulty: $c.tasks.difficulty,
      estimate: $c.tasks.estimate,
      kinds: $c.tasks.fields.kind,
      "link-kinds": $c.tasks.links.kinds,
      comments: $c.tasks.comments,
      extensions: extensions($c.tasks),
      tags: $c.tasks.tags,
      "derive-tags": $c.tasks."derive-tags",
      collectors: $c.tasks.collectors,
      waits: $c.tasks.waits,
      rules: labelled("tasks.rules"; ["add", "flip"]; $c.tasks.rules),
      body: {
        headers: labelled("tasks.body.headers"; ["add", "flip"]; $c.tasks.body.headers.rules),
        content: labelled("tasks.body.content"; ["add", "flip"]; $c.tasks.body.content.rules),
        words: $c.tasks.body.content.words
      }
    },
    templates: $c.templates,
    views: $c.views
  };

if . == null then dropped
elif has("config") then report(.config)
else report(.)
end
