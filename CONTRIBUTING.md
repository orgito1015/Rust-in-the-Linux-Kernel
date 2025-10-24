# Contributing Guide

Thanks for helping! Here’s how to contribute:

## 1) Issues
- Use templates under **New issue →** pick *Research Task*, *Doc Improvement*, or *Bug Report*.
- Small suggestions? Use a single PR; larger proposals → open a *Research Task* first.

## 2) Branch & PR
- Create a feature branch: `feat/<short-topic>` or `docs/<area>`.
- Keep PRs focused and small. Link related issues with `Fixes #123`.
- Run CI locally if possible: `./scripts/check-links.sh` and markdown lint (CI will run anyway).

## 3) Style
- Markdown with clear headings.
- Prefer links to primary sources (LKML threads, kernel docs, conference talks).
- Use fenced code blocks for shell/Rust/C snippets with language tags.

## 4) Where to add content
- **docs/** for curated, stable knowledge.
- **research/** for raw notes, timelines, and reference dumps.
- **snippets/** for small Rust/C/kbuild examples (keep minimal & documented).

## 5) Licensing
- By contributing, you agree your contributions are licensed under MIT.
