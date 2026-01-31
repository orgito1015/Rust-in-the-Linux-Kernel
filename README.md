![Tux the Linux penguin with Rust logo on chest against green matrix code background](./Rust%20in%20the%20Linux%20kernel.png)

# Rust in the Linux Kernel — Open Research

[![CI](https://img.shields.io/github/actions/workflow/status/orgito1015/Rust-in-the-Linux-Kernel/ci.yml?label=CI)](./.github/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Good First Issue](https://img.shields.io/github/issues/orgito1015/Rust-in-the-Linux-Kernel/good%20first%20issue.svg)](https://github.com/orgito1015/Rust-in-the-Linux-Kernel/labels/good%20first%20issue)

A community-driven, structured research project on **Rust in the Linux kernel**: motivation, history, technical
integration, toolchains, contribution paths, challenges, and future directions.

**Status**: Active research project tracking Rust's integration into Linux mainline (6.1+) through 2026.

## Project Status (2026)

- 📚 **Comprehensive Documentation**: 8 detailed documents covering all aspects of Rust-for-Linux
- 🔬 **Active Research**: Tracking developments from Linux 6.1 (2022) through 6.13+ (2026)
- 📊 **Growing Content**: Code snippets, academic references, and research notes
- 🌱 **Community-Driven**: Open to contributions from developers and researchers worldwide

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
git clone https://github.com/orgito1015/Rust-in-the-Linux-Kernel.git
cd Rust-in-the-Linux-Kernel
./scripts/setup-dev.sh
```

## Contributing
We welcome contributions of all kinds—notes, links, corrections, code snippets.  
Read [CONTRIBUTING.md](./CONTRIBUTING.md) and our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Project Layout
- `docs/` — curated, high-level documents (8 comprehensive guides)
- `research/` — raw notes, references, and exploratory work
  - `notes/` — research observations and analysis
  - `references/` — academic papers and bibliography
  - `snippets/` — code examples and demonstrations
- `scripts/` — helper scripts (link checks, setup)
- `.github/` — issues templates, PR template, CI workflows

## Documentation

### Core Documents

1. **[Introduction & Motivation](./docs/01-intro-and-motivation.md)** — Why Rust for the kernel?
2. **[Historical Background](./docs/02-history.md)** — Timeline from 2019 through 2026
3. **[Technical Overview](./docs/03-technical-overview.md)** — Kbuild integration and FFI
4. **[Toolchain & Dependencies](./docs/04-toolchain-and-deps.md)** — Build requirements
5. **[Development & Contribution](./docs/05-dev-and-contrib.md)** — How to get started
6. **[Challenges & Limitations](./docs/06-challenges-and-limits.md)** — Current obstacles
7. **[Future Directions](./docs/07-future-directions.md)** — What's next?
8. **[Resources & References](./docs/08-resources.md)** — Links, talks, and learning materials

## License
MIT — see [LICENSE](./LICENSE).
