<!-- Generated from docs/src/contributing/documentation.typ by `just docs md`. Do not edit; edit the source. -->

# Writing the documentation

The doc gates catch mechanical errors:

- `just docs check` finds a dead link.
- `just docs facts` finds a derived fact with no entry.
- `just docs recipes` finds a hand-typed recipe name.
- `just docs prose` finds history words, shouting, an em dash in text a reader sees and a typed name.
- `just docs blocks` finds a hand-typed code block.

No gate finds duplicated prose or a page that doesn’t help its reader. The rules below cover what the gates can’t.

The span, recipe, prose and block gates read the Typst sources through one extractor in `.just/scripts/_doc-blocks-lib.sh`. It parses each file with `typstyle --ast` and prints each inline raw, block raw line, comment line and prose line with its file and line, so a fence inside a string hides nothing. The same file lists the sources the gates read, and fails when an entry matches no file.

The span gate hands each `mt` command line it finds in a span to `mt _cli --check-argv`. It finds them after a `$ ` prompt, `VAR=value` words or `&&`. That runs the `@name` rewrite, clap and the status-verb parser that `mt` itself runs, so the gate holds no copy of the command grammar. A missing required argument passes, because a span shows part of a command.

It reads a config key span, such as `[ls] hyperlinks`, against `schema/config.schema.json`, or against `schema/at.schema.json` when the span’s first segment is an `at.toml` key such as `[board]` or `[at.<name>]`. A key that neither schema has fails with the closest one.

## Where the docs live

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

A generated fact (`www/generated/*.json`, read by both render paths) can’t drift. A hand-written sentence can. Two pages state the same fact in different words, one of them changes, and no gate fails. So state a fact on one page and link to it from the others. Content that both render paths show, like the tagline, the recipe table and the Typst section in README and on the site root, lives once in a module both call (`www/utils/demo.typ`).

The `just docs dup-prose` audit reports two kinds of repeat that are deliberate:

- A document an agent reads whole (AGENTS.md, the skill and its references) keeps its own copy of what it needs, because its reader may have no site access.
- Shared code, such as a render loop or a crate table, may be extracted or copied. Never paraphrase it, because reworded code is worse than either.

## Use checked examples

A code block in the docs comes from a real file under `examples/` that `mt check` validates, from a capture of a real run, or from a derived fact. A hand-typed snippet goes stale when the code changes, and the reader can’t tell.

- To show a fixture, mark the lines between `// docs:slice <id>` and `// docs:end`. `www/generate-examples.sh` cuts them into `www/generated/`, and the page renders them with `www/utils/slices.typ`.
- The DSL page’s field table is `lib/prelude.typ`’s own signature.
- To show a run, use a capture. `www/capture.sh` writes a record of each command the build ran in a terminal, what it printed and its exit code. A page shows the record with `terminal(<slug>)`, or `commands(<slug>)` for the commands alone. Both print each command from the record, so a page can’t show a command that didn’t run.

`exit-code(<slug>)` prints the exit code of a refusal the page shows, the last command that exited non-zero. For a capture with two refusals, pick one with `step: <n>`, the command’s index from 0. The build fails when that command passed or doesn’t exist. A step that exits with an unexpected code fails the build. Every refusal exits 1 and every usage error 2, so a refusal capture also names its reason with `--expect <regex>`, and the build fails unless the output, without its color, matches it. It also fails on a non-zero `--exit` with no `--expect`.

To quote one line of what a capture printed, such as an `error:` or `hint:` line, write `capture-line(<slug>, <regex>)`. It renders the first line of the capture that matches the regex, and the build fails when none does. The span gate fails on an inline span that starts with `error:`, `warning:`, `hint:` or `note:` followed by text, because a hand-typed copy goes stale when the message changes.

`just docs blocks` fails on a fenced block, or a block `raw(...)`, whose text is written in the source. If no fixture can hold a block, such as a grammar sketch, put `// docs:hand-typed <reason>` on the line above it. The gate lists the sources not yet converted. A listed source with no hand-typed block left fails, so the list only shrinks.

## Voice

Write the way an engineer explains a tool to a peer: say “you”, use contractions, open with what the reader wants to do, show the command and what it prints, and say what a choice costs. Don’t assume a team, because the reader may work alone, with agents, or both.

- Bad: “Each task is a Typst file in your repository. `mt` checks every task and commit against the rules your team writes in `.mindtape/config.toml`.”
- Good: “Your tasks live in the repo, one Typst file each. Rules go in `.mindtape/config.toml`, and `mt` refuses any edit or commit that breaks them.”

Each kind of reader gets its own register:

- **The site, README, `CONTRIBUTING.md` and the `docs/contributing/*.md` deep dives** use the voice above.
- **`--help` text** says in one or two sentences what the flag does, from the user’s side. It leaves out internals such as config tables the user never sees.
- **An error message** names what failed, quotes the value that failed, and says what to try next on a `hint:` line when there’s an obvious next step. It’s one line that starts with `error:`. Build it with the `Diag` builder in `crates/mindtape-cli/src/diag/`, whose module docs hold the format.
- **AGENTS.md, SKILL.md and the skill’s references** are read by agents, any agent and not one vendor’s. They’re imperative: one situation, one action, the exact command. They state the rules an agent must follow, such as making changes through `mt` commands, which a human page leaves as a choice.

## Plain prose

Write for a busy engineer who reads each sentence once. The rules apply to comments in doc sources too: a comment says what the code does and why, in as few lines as it takes.

1. **State the rule and its reason.** A reader who knows the reason can apply the rule to a case it doesn’t name.
   - Bad: “Keep the recipe and the mmz `examples` scope in step.”
   - Good: “Keep the recipe and the mmz `examples` scope in step, because mmz re-runs this gate only when a file in that scope changes.”
2. **Leave out history.** The reader needs today’s behavior, and `git log` holds how it got there.
   - Bad: “the word-splitting that `[positional-arguments]` used to guard that recipe against, refused at dispatch now instead.”
   - Good: “a value containing a space, which the pattern refuses at dispatch, so the recipe never word-splits it.”
3. **Use the literal word.** A reader has to translate a figure of speech, and two readers translate it differently.
   - Bad: “`.just/docs.just`’s currently userless one is load-bearing and cannot be tidied away”
   - Good: “`.just/docs.just` keeps its own even though none of its recipes uses it yet”
4. **Make the actor the subject.** Files, fields and values don’t know, want or own anything. Name the program or person that acts.
   - Bad: “against what git knows”
   - Good: “against the TOML files git lists, tracked or not yet added”
5. **Drop the “not Y” half when no reader asked about Y.** It makes the reader hold Y in mind only to discard it.
   - Bad: “this recipe is only the enforcement point, not a separate lint surface.”
   - Good: “this recipe runs them.”
6. **Put one idea in each sentence.** A reader takes in one idea at a time, so split a sentence at its semicolons and at a colon that announces a claim.
   - Bad: “Editing any source stales the lcov, so it skips on the commits that matter; it catches the free case only.”
   - Good: “It runs only against a fresh lcov. Editing any source makes the lcov stale, so it skips on the commits that matter.”
7. **Don’t shout.** All-caps words and adverbs like “crucially” add emphasis. Emphasis on many lines teaches the reader to skip it.
   - Bad: “An unknown bare name is refused naming what IS available”
   - Good: “An unknown bare name is refused with a list of the available packs”

`just docs prose` enforces two of these rules and two of its own. It fails on the history words `now`, `used to`, `no longer`, `anymore` and `formerly`, on an all-caps word outside its acronym list, on an em dash in text a reader sees, and on the product’s name typed out.

- A Typst source writes the name as `#mindtape`, which `www/utils/site.typ` reads from the site title in `www/tola.toml`, so a typed name fails anywhere in the file, raw spans and comments included. A Markdown source can’t reach the constant, so there only the old CamelCase spelling fails.
- A shell constant passes only as `$NAME`, `${NAME}` or `NAME=`. A lone capital beside an all-caps word counts as shouting, so wrap a constant’s bare name in backticks.
- An em dash fails in the rendered prose and the string literals of `www/content` and `www/utils`, in every schema `description` and in every help string of `mt _cli --json`, and the hit names the file and line, or the key path. A code comment, AGENTS.md, the skill and the page build errors in `www/utils/page.typ` and `site.typ` are not that text, so they may keep one. Split the sentence, or use a comma, a colon in a list or parentheses.
- It skips raw spans, fenced blocks and double-quoted text, so a quoted bad example passes.
- It applies the same rules to the full-line `#` comments of every shell script, YAML file, Justfile and `.just` module that git lists, tracked or not yet added, and to every `description` in `schema/config.schema.json` and `schema/at.schema.json`. So it reads a new script the day it lands, and a new violation fails the gate outright.

The word lists are in `.just/scripts/_doc-prose-lib.sh`, and what the scan skips is argued in `.just/scripts/check-doc-prose.sh`.

## The duplicate-prose audit

`just docs dup-prose` reads the doc sources, strips markup and code, and reports sentences that two files share, worst pair first. It’s advisory, because the threshold is a judgment call and you decide where each fact lives. Run it before and after a refactor. The algorithm, its settings and why it’s written in jq are in `contrib/dup-prose.jq`.
