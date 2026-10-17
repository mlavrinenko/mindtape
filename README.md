<!-- Generated from docs/src/readme.typ by `just docs md`. Do not edit; edit the source. -->

<h1 align="center"><img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg" alt="Mindtape" width="120" /><br />
 Mindtape</h1>

<h4 align="center">Tasks as code. One Typst file per task, checked against your own rules. Write rules in <code>.mindtape/config.toml</code>, and <code>mt check</code> runs them over every task and commit.</h4>

<h4 align="center"><a href="https://mlavrinenko.github.io/mindtape/">Documentation</a> · <a href="https://mlavrinenko.github.io/mindtape/quickstart/">Quickstart</a></h4>

---

Here's a real task.

[`examples/api-service/tasks/throttle-failed-logins-per-ip.typ`](https://github.com/mlavrinenko/mindtape/blob/main/examples/api-service/tasks/throttle-failed-logins-per-ip.typ)

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Throttle failed logins per IP",
  kind: "feat",
  priority: ice(impact: 7, confidence: 0.7, ease: 4),
  status: proposed(2026, 9, 28),
  estimate: (src-files: 2),
  links: depends-on("add-redis-client.typ"),
)

== Problem
+ Run `for i in $(seq 50); do curl -so /dev/null -w '%{http_code} ' -d user=ada -d password=x localhost:8080/login; done`.
+ 🔴 It prints `401` fifty times. A script can guess passwords as fast as the server answers.

== TO-BE
- 🟢 After 10 failed logins from one IP in 10 minutes, `/login` returns `429` with a `Retry-After` header.

== Changes
- `src/auth/limiter.rs`, new: count failures per IP in Redis, each count expiring after 10 minutes.
- `src/auth/login.rs`: ask the limiter before checking the password, and record each failure.

== Alternatives
- A counter in process memory. Each replica would count on its own, so three replicas would allow 30 tries.

== Out of scope
- Locking accounts by username, which needs its own unlock flow.
```

## Bring your own rules

None of these checks ship with Mindtape. Each one is a few lines of config. Here’s what `mt` says when a task breaks one.

| Recipe | What it does | What `mt` prints |
| --- | --- | --- |
| [Split oversized tasks](https://mlavrinenko.github.io/mindtape/split-oversized-tasks/) | Cap how many source files a task may touch when it takes judgment, so a big design gets split before it’s filed. | `error: <new task from template 'task'>:2: rules.judgment-stays-small: split the decision from the mechanical edits, so each task touches 5 source files or fewer` |
| [Require a task shape](https://mlavrinenko.github.io/mindtape/require-a-task-shape/) | Refuse a task that doesn’t say how to reproduce the problem, what done looks like and which files change. | `error: missing required slots: 'to-be', 'changes', 'out-of-scope'` |
| [Start titles with a verb](https://mlavrinenko.github.io/mindtape/start-titles-with-a-verb/) | Refuse a task whose title doesn’t start with a verb, so each line of a list says what the task does. | `error: <new task from template 'task'>:2: rules.imperative-title: start the title with an imperative verb` |
| [Tie work to your goals](https://mlavrinenko.github.io/mindtape/tie-work-to-goals/) | Make each milestone say how it serves your goals, and refuse one that helps none of them. | `error: <new task from template 'milestone'>:3: rules.goals: it helps no goal, so name one it helps or drop it` |
| [Limit work in progress](https://mlavrinenko.github.io/mindtape/limit-work-in-progress/) | Allow one milestone and one task in progress, and refuse to start a task before the tasks it depends on are done. | `error: tasks/add-redis-client.typ:2: rules.one-task: finish the task in progress before you start another` |
| [Wait on upstream](https://mlavrinenko.github.io/mindtape/wait-on-upstream/) | Hold a task until a command says the release it needs has landed, and free it the moment it has. | `error: status requires an unblocked task: 'wip'` |
| [Trace commits to tasks](https://mlavrinenko.github.io/mindtape/trace-commits-to-tasks/) | Refuse a commit that names no task, then ask which tasks changed a file before you edit it. | `error: .git/COMMIT_EDITMSG: commits.rules.work-names-a-task: name the task this commit works on in a` `Refs: <stem>` `footer` |

## The elephant in the room: Typst vs Markdown + frontmatter

Why pick [Typst](https://typst.app/) over Markdown to power a local task tracker? Here are a few reasons.

1. **Language server autocompletion.** `task` declares every field as a named parameter, so your editor completes field names and flags a misspelled one.
2. **Shared helpers.** `#import` your own Typst module to fill tasks the way you like.
3. **Computed fields.** Fill a field with an expression, like `due: release - duration(days: 14)`. Change `release` in your shared module, and every deadline moves.
4. **Custom validators.** Bring your own checks by writing a Typst function. See [Self-validating tasks](https://mlavrinenko.github.io/mindtape/self-validating-tasks/).

## Install

```sh
cargo install mindtape-cli
```

[Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) covers prebuilt binaries, `nix run` and a first project from `mt init` to `mt check`.

## Documentation

- **Start**: [Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) · [Read many projects](https://mlavrinenko.github.io/mindtape/read-many-projects/) · [Concepts](https://mlavrinenko.github.io/mindtape/concepts/) · [Comparison](https://mlavrinenko.github.io/mindtape/comparison/)
- **Tasks**: [Task files](https://mlavrinenko.github.io/mindtape/task-files/) · [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) · [Task links](https://mlavrinenko.github.io/mindtape/links/) · [Recurring tasks](https://mlavrinenko.github.io/mindtape/recurring/) · [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) · [Derived tags](https://mlavrinenko.github.io/mindtape/derived-tags/)
- **Fields**: [Statuses](https://mlavrinenko.github.io/mindtape/statuses/) · [Priority](https://mlavrinenko.github.io/mindtape/priority/) · [Estimates](https://mlavrinenko.github.io/mindtape/estimates/) · [Views & dictionaries](https://mlavrinenko.github.io/mindtape/named-sets/)
- **Checks**: [Rules](https://mlavrinenko.github.io/mindtape/rules/) · [Rule packs](https://mlavrinenko.github.io/mindtape/rule-packs/) · [Commit rules](https://mlavrinenko.github.io/mindtape/commit-rules/) · [Flip hooks](https://mlavrinenko.github.io/mindtape/hooks/)
- **Reference**: [CLI reference](https://mlavrinenko.github.io/mindtape/cli/) · [Filter language](https://mlavrinenko.github.io/mindtape/filters/) · [Configuration](https://mlavrinenko.github.io/mindtape/config/) · [Canonical schema](https://mlavrinenko.github.io/mindtape/schema/)
- **Integrate**: [Self-validating tasks](https://mlavrinenko.github.io/mindtape/self-validating-tasks/) · [For AI agents](https://mlavrinenko.github.io/mindtape/agents/) · [Import & export](https://mlavrinenko.github.io/mindtape/import-export/) · [Internals](https://mlavrinenko.github.io/mindtape/internals/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE).
