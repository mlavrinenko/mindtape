# MindTape

A file-based task tracker powered by [Typst](https://typst.app/).

Write `.typ` files to manage tasks across your projects. MindTape watches your folders,
evaluates the Typst files, and indexes everything into a searchable database.

## Why Typst?

Unlike Markdown, Typst files can import each other, define typed variables, and be
statically checked. This means your task data is structured, validated, and composable.

```typ
#import "@mind-tape": due, tag

= Sprint 12

- [ ] implement auth #due(datetime(year: 2026, month: 3, day: 1)) #tag("backend")
- [x] design mockups #tag("design")
- [ ] write tests

#let note = "blocked on API spec from team B"
```

## How It Works

1. Configure folders to watch
2. Run `mind-tape watch`
3. MindTape evaluates your `.typ` files using the Typst compiler
4. Tasks, properties, and bindings are indexed into SQLite
5. Query with `mind-tape list`, filter by status/tag/due date

## Status

Early development. See [docs/ROADMAP.md](docs/ROADMAP.md) for the plan.

## Docs

- [Design](docs/DESIGN.md) — architecture, data model, technical decisions
- [Roadmap](docs/ROADMAP.md) — MVP scope, milestones, non-goals
