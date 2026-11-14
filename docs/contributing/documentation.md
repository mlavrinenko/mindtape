<!-- Generated from docs/src/contributing/documentation.typ by `just docs md`. Do not edit; edit the source. -->

# Writing the documentation

The doc gates catch mechanical errors:

- `just docs check` finds a dead link.
- `just docs facts` finds a derived fact with no entry.
- `just docs recipes` finds a hand-typed recipe name.
- `just docs vale-prose` holds Typst prose to the Vale style.
- `just docs prose` finds history words, shouting, an em dash or a typed name in Markdown, comments, schema descriptions and help text, where Vale does not read.
- `just docs blocks` finds a hand-typed code block.

No gate finds duplicated prose or a page that doesn’t help its reader. The rules below cover what the gates can’t.

The span, recipe, prose and block gates read the Typst sources through one extractor in `.just/scripts/_doc-blocks-lib.sh`. It parses each file with `typstyle --ast` and prints each inline raw, block raw line, comment line and prose line with its file and line. A fence inside a string then hides nothing. The same file lists the sources the gates read and fails on an unmatched entry.

The span gate passes each `mt` command line it finds, after a `$ ` prompt, `VAR=value` words or `&&`, to `mt _cli --check-argv`. That runs the `@name` rewrite, clap and the status-verb parser `mt` itself runs. The gate reuses the command grammar instead of copying it. A missing required argument passes, because a span shows part of a command. It reads a config key span such as `[ls] hyperlinks` against `schema/config.schema.json`, or `schema/at.schema.json` when its first segment is an `at.toml` key such as `[board]`. A key that neither schema has fails with the closest one.

## Doc sources

- `www/content/**/*.typ` is the docs site, and `www/utils/*.typ` holds the notes its generated references render.
- Each `mt` command has a page, `www/content/cli/<command>.typ`, which renders the command from `www/utils/cli-ref.typ` and can add prose of its own. `just docs coverage` fails on a command with no page or no `www/utils/cli-notes.typ` entry.
- `docs/src/readme.typ`, `docs/src/agents.typ` and `docs/src/skill.typ` render to README.md, AGENTS.md and `skills/mindtape/SKILL.md`.
- `docs/src/contributing.typ` renders CONTRIBUTING.md, and `docs/src/contributing/*.typ` render `docs/contributing/*.md`.

Edit the source, never the render: `just docs md` overwrites it.

## Rewrite the whole file

Before you change a file, decide who reads it and what they can do after. Then rewrite the whole file for that reader. A file can pass every gate and still waste its reader’s time, because each paragraph that stayed was cheaper to keep than to delete.

A file at its line cap covers too much, and trimming words to fit doesn’t fix that. Rewrite it whole, or move the argument to a `docs/contributing/*.md` file and leave a link in the always-loaded source.

## Fix the bug, not the page

The docs never mention a bug or a workaround. When writing a page turns up behavior that would need a caveat, fix the behavior and describe it fixed.

## State each fact once

A generated fact (`www/generated/*.json`, read by both render paths) can’t drift. A hand-written sentence can. Two pages state the same fact in different words, one of them changes, and no gate fails. State a fact on one page and link to it from the others. Content that both render paths show lives once in a module both call (`www/utils/demo.typ`). That covers the tagline, the recipe table and the Typst section in README and on the site root.

The `just docs dup-prose` audit reports two kinds of repeat that are deliberate:

- A document an agent reads whole (AGENTS.md, the skill and its references) keeps its own copy of what it needs, because its reader may have no site access.
- You may move shared code into one place, such as a render loop or a crate table. Never paraphrase it, because reworded code is worse than either.

## Use checked examples

A code block in the docs comes from a real file under `examples/` that `mt check` validates, from a capture of a real run, or from a derived fact. A hand-typed snippet goes stale when the code changes, and the reader can’t tell.

- To show a fixture, mark the lines between `// docs:slice <id>` and `// docs:end`. `www/generate-examples.sh` cuts them into `www/generated/`, and the page renders them with `www/utils/slices.typ`.
- The DSL page’s field table is `lib/prelude.typ`’s own signature.
- To show a run, use a capture. `www/capture.sh` writes a record of each command the build ran in a terminal, what it printed and its exit code. A page shows the record with `terminal(<slug>)`, or `commands(<slug>)` for the commands alone. Both print each command from the record, so a page can’t show a command that didn’t run.

`exit-code(<slug>)` prints the exit code of a refusal the page shows, the last command that exited non-zero. For a capture with two refusals, pick one with `step: <n>`, the command’s index from 0. The build fails when that command passed or doesn’t exist. A step that exits with an unexpected code fails the build. Every refusal exits 1 and every usage error 2. A refusal capture gives its reason with `--expect <regex>`, and the build fails unless the output, without its color, matches it. It also fails on a non-zero `--exit` with no `--expect`.

To quote one line of what a capture printed, such as an `error:` or `hint:` line, write `capture-line(<slug>, <regex>)`. It renders the first line of the capture that matches the regex, and the build fails when none does. The span gate fails on an inline span that starts with `error:`, `warning:`, `hint:` or `note:` followed by text, because a hand-typed copy goes stale when the message changes.

`just docs blocks` fails on a fenced block, or a block `raw(...)`, whose text you write in the source. If no fixture can hold a block, such as a grammar sketch, put `// docs:hand-typed <reason>` on the line above it. The gate lists the sources not yet converted. A listed source with no hand-typed block left fails. The list only shrinks.

## Voice

Write the way an engineer explains a tool to a peer. Say “you”, use contractions, open with what the reader wants to do, and show the command and what it prints. Say what a choice costs. Don’t assume a team, because the reader may work alone, with agents, or both.

- Bad: “Each task is a Typst file in your repository. `mt` checks every task and commit against the rules your team writes in `.mindtape/config.toml`.”
- Good: “Your tasks are Typst files in the repo, one per task. Rules go in `.mindtape/config.toml`, and `mt` refuses any edit or commit that breaks them.”

Each kind of reader gets its own register:

- **The site, README, `CONTRIBUTING.md` and the `docs/contributing/*.md` deep dives** use the voice above.
- **`--help` text** says in one or two sentences what the flag does, from the user’s side. It leaves out internals such as config tables the user never sees.
- **An error message** states what failed, quotes the value that failed, and gives the next step on a `hint:` line when there’s an obvious one. It’s one line that starts with `error:`. Build it with the `Diag` builder in `crates/mindtape-cli/src/diag/`, whose module docs hold the format.
- **AGENTS.md, SKILL.md and the skill’s references** target any agent, not one vendor’s. They’re imperative: one situation, one action, the exact command. They state the rules an agent must follow, such as making changes through `mt` commands, which a human page leaves as a choice.

## Plain prose

Write for a busy engineer who reads each sentence once. A comment states what the code does and why, in as few lines as it takes.

### The Vale gate

`just docs vale-prose` runs Vale over `docs/src`, `www/content` and `www/utils`, using the style in `.config/vale.ini`. The dev shell builds the rule set from pinned flake inputs through `nix/dev/vale-styles.nix` and links it at `target/vale/styles`. Every rule sits at `error`, so one alert fails the gate. The rules come from Microsoft, Google, write-good, the ai-tells package and this repo’s own `.config/vale/local/`. The prose gates and the code-comment gate call `vale` 3.27. One binary reads comments and Typst prose, and `nix/dev/shell.nix` builds it.

A new rule starts at `NO`. Clear its alerts over the docs, then move its level to `error` in `.config/vale.ini`. The level lives in the config, not in the rule file. The build strips the upstream `level`, so a fetched rule cannot set one. When the prose can’t satisfy a rule, keep it at `NO` and write the reason beside its level block.

Every level has a check. `just lint vale-levels` fails on a `MinAlertLevel` other than `error`, on any rule set to `warning`, and on a rule of an enabled style with no level line. It reads the rule names from `target/vale/styles`. A new rule needs a line in every config that enables its style.

### Rules the prose follows

Every rule sits at `error`. The example shows what the rule flags.

#### Sentence shape

| Rule | Flags | Example |
| --- | --- | --- |
| `Prose.SentenceLength` | more than 30 words | “one clause, then a second, then a third that runs the sentence past the limit” |
| `Prose.PassiveVoice` | a passive clause | “the file is written” |
| `Prose.Semicolon` | a semicolon | “a; b” |
| `Prose.ConsequenceClause` | a `so` clause | “the loader sorts, so the first entry wins” |
| `Prose.EmDash` | an em dash character | “the rule — and its reason” |
| `Prose.DoubleHyphen` | a double hyphen outside code | `--verbose` |
| `Prose.Latin` | a Latin abbreviation | “e.g., i.e., etc.” |
| `Prose.HeadingPunctuation` | end punctuation in a heading | “How it works.” |
| `Prose.HeadingCase` | Title Case in a heading | “How It Works” |

#### Word choice

| Rule | Flags | Example |
| --- | --- | --- |
| `Prose.Weasel` | an unquantified qualifier | “several or clearly” |
| `Prose.Wordiness` | a longer phrase for a shorter one | “in order to” |
| `Prose.FormalRegister` | a formal synonym | “utilize” |
| `Prose.OverusedVocabulary` | an AI-favored word | “delve” |
| `Prose.JourneyMetaphors` | a progress metaphor | “the journey” |
| `Prose.AICompoundPhrases` | a stock phrase | “a rich tapestry” |
| `Prose.UrgencyInflation` | invented stakes | “cannot be overstated” |
| `Prose.PromotionalPuffery` | a stock scenic phrase | “nestled in the heart of” |
| `Prose.SycophancyMarkers` | flattery | “great question” |
| `Prose.HedgingPhrases` | throat-clearing | “it’s worth noting” |
| `Prose.FillerPhrases` | a phrase you can delete | “at the end of the day” |
| `Prose.OpeningCliches` | a generic opener | “In today’s fast-paced world” |
| `Prose.ClosingPleasantries` | a sign-off | “Happy coding” |
| `Prose.ConclusionMarkers` | a conclusion announcement | “In conclusion” |
| `Prose.RestatementMarkers` | a restatement frame | “In other words” |
| `Prose.StructureAnnouncements` | a narration of the page | “Let’s dive in” |
| `Prose.UnpackExplore` | an explainer prompt | “let’s unpack” |
| `Prose.VagueAttributions` | an unnamed source | “experts say” |
| `Prose.FalseExclusivity` | insider framing | “not just a tool” |
| `Prose.FormalTransitions` | an essay connector | “Moreover” |

#### Sentence frames

| Rule | Flags | Example |
| --- | --- | --- |
| `Prose.ServesAsDodge` | an inflated copula | “serves as a” |
| `Prose.PseudoCleft` | a `What ... is` frame | “What matters is the order” |
| `Prose.ShellNounCopula` | a shell noun | “the problem is that” |
| `Prose.LabelAndExplain` | a label and a colon | “The result: the gate fails” |
| `Prose.ExplainerLeads` | an announcement of what follows | “Here’s how it works” |
| `Prose.NegatedObject` | a shifted `no` | “allows no override” |
| `Prose.NegatedSubject` | an absence as subject | “No rule reads it” |
| `Prose.BareNames` | a designation verb | “the field names the target” |
| `Prose.AnthropomorphicCognition` | cognition given to a tool | “the gate knows” |
| `Prose.AnthropomorphicJustification` | volition given to a tool | “because the code wants” |
| `Prose.ShipOveruse` | `ship` as a verb | “ships with” |
| `Prose.ExplainerHeadings` | a question heading | “Why a hook fires” |

#### Names and spelling

| Rule | Flags | Example |
| --- | --- | --- |
| `Prose.History` | an old state | “no longer” |
| `Prose.Shouting` | all caps | “MUST” |
| `Prose.ProductName` | a typed product name | “Mind tape” |
| `Prose.Spelling` | an unknown word | “retreive” |
| `Vale.Terms` | a wrong case for a name | “Github” |
| `Vale.Avoid` | a rejected spelling | “E-mail” |
| `Vale.Repetition` | a word typed twice | “the the” |

### Task bodies

`just docs vale-tasks` runs the same style over `truth/tasks`. Every task body passes the same 48 rules as a docs page, with no month or task section in `.config/vale.ini`. `Prose.HeadingCase` lists `AS-IS` and `TO-BE` as terms, because `mt check` matches those headings literally. That arm carries no `gate` tag, since `mt flip` writes the task file it would read.

### Code comments

`just lint banners` runs `vale` over every tracked Rust, Shell and CSS file. Vale reads a comment through its tree-sitter grammar. The rule sees the comment and never the code. `Sep.Banner` fails a comment whose text ends in five or more `-`, `=`, `*`, `_`, `~` or `#` characters. Delete the run and keep any label it frames. The dev shell exports the pinned grammars from `nix/dev/vale-grammars.nix`, and the gate runs offline.

`just lint vale-code` holds Rust doc comments to the prose rules. `.config/vale-code.ini` gives `crates/` to Vale with the `RustDoc` View, a tree-sitter query that keeps `///` and `//!` and drops an inline `//` comment. `[formats] rs = md` parses each comment body as Markdown. A code span and a doc test stay out. `Prose.History` and `Prose.Shouting` sit at `error`. Every other `Prose` rule is `NO` until a backfill task moves it to `error`. Shell comments join later through the `[*.sh]` section the config carries.

### Prose gate coverage

`just docs prose` keeps four older checks for Markdown, the full-line comments of every tracked shell script, YAML file and Justfile, both schema descriptions and every `mt _cli --json` help string. It also checks a shell constant, written as `$NAME`, `${NAME}` or `NAME=`, and a lone capital beside an all-caps word. A quoted example passes in both gates. The word lists are in `.just/scripts/_doc-prose-lib.sh`, and `.just/scripts/check-doc-prose.sh` argues what the scan skips.

The rest needs a reader’s judgment:

1. State the rule and its reason, so a reader can apply it to another case.
2. Use the literal word: a figure of speech reads differently to each reader.
3. Make the actor the subject: the program or person that acts, not the file or field.
4. Drop the “not Y” half when no reader asked about Y.
5. Put one idea in each sentence: split at a semicolon or a colon that announces a claim.
6. Open with the fact, not a question or a staged scene.
7. Keep the fact at its real size: say what a thing does, not that it is the whole gate.
8. Stop when the point stands. Delete a closing sentence that repeats the section.
9. Use a list for parallel cases: three or more cases in one sentence are hard to scan.

## The duplicate-prose audit

`just docs dup-prose` reads the doc sources, strips markup and code, and reports sentences that two files share, worst pair first. It’s advisory, because the threshold is a judgment call and you decide where each fact belongs. Run it before and after a refactor. The algorithm, its settings and why it’s written in jq are in `contrib/dup-prose.jq`.
