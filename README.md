# Rust in the Linux Kernel — Open Research

[![CI](https://img.shields.io/github/actions/workflow/status/OWNER/rust-in-linux-kernel-research/ci.yml?label=CI)](./.github/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Good First Issue](https://img.shields.io/github/issues/OWNER/rust-in-linux-kernel-research/good%20first%20issue.svg)](https://github.com/OWNER/rust-in-linux-kernel-research/labels/good%20first%20issue)

A community-driven, structured research project on **Rust in the Linux kernel**: motivation, history, technical integration, toolchains, contribution paths, challenges, and future directions.

## Goals
- Produce a **clear, citable knowledge base** (docs/) about Rust-for-Linux.
- Maintain **living roadmap** of research tasks (see [ROADMAP.md](./ROADMAP.md)).
- Provide **reproducible snippets** that show Rust↔C interoperability in kernel context.

## Who is this for?
- Kernel & systems developers curious about Rust.
- Researchers compiling references and timelines.
- Contributors wanting a friendly, structured starting point.

## Quick Start
```bash
git clone https://github.com/OWNER/rust-in-linux-kernel-research.git
cd rust-in-linux-kernel-research
./scripts/setup-dev.sh
```

## Contributing
We welcome contributions of all kinds—notes, links, corrections, code snippets.  
Read [CONTRIBUTING.md](./CONTRIBUTING.md) and our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Project Layout
- `docs/` — curated, high-level documents
- `research/` — raw notes, references, and exploratory work
- `scripts/` — helper scripts (link checks, setup)
- `.github/` — issues templates, PR template, CI workflows

## License
MIT — see [LICENSE](./LICENSE).
