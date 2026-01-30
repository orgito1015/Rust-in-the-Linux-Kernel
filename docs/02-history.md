# Historical Background

## Early Proposals and LKML Debates

### Initial Discussions (2019-2020)

The idea of using Rust in the Linux kernel wasn't entirely new, but serious discussions began around 2019:

- **2019**: Early experimental work by various developers exploring Rust bindings
- **April 2020**: Initial discussions on LKML about the feasibility of Rust in kernel
- **Community reactions**: Mixed responses ranging from enthusiastic support to skeptical concerns

### Key Debates

The Linux Kernel Mailing List (LKML) debates centered on several topics:

1. **Performance concerns**: Would Rust add overhead?
2. **Toolchain stability**: Is Rust mature enough for kernel development?
3. **Maintenance burden**: Who will maintain Rust infrastructure?
4. **Learning curve**: Will this fragment the developer community?

## Rust-for-Linux Project Emergence

### Project Launch (2020-2021)

- **July 2020**: Miguel Ojeda begins serious work on Rust infrastructure
- **Key maintainers**:
  - Miguel Ojeda (project lead)
  - Alex Gaynor (initial contributor)
  - Wedson Almeida Filho (core abstractions)
  - Gary Guo (compiler expertise)
  - Geoffrey Thomas (early contributor)

### Community Support

Support came from various organizations:

- **Google**: Funding and engineering resources
- **ARM**: Interest in safe driver development
- **Microsoft**: Learning from their Rust experience
- **Red Hat**: Evaluating for enterprise use

### Initial Milestones

- **April 2021**: RFC v1 posted to LKML with basic infrastructure
- **October 2021**: RFC v2 with improved abstractions and examples
- **December 2021**: RFC v3 addressing community feedback
- **April 2022**: RFC v4 nearing merge-ready state

## Major Milestones

### Linux 6.1 (December 2022) - Initial Rust Support

**Historic Achievement**: First kernel release with Rust support

- Merged experimental Rust support infrastructure
- Basic abstractions over kernel APIs
- Sample Rust driver included
- Requires `CONFIG_RUST=y` to enable
- **Minimum Rust version**: 1.62.0

**What was included**:
- Core infrastructure (`rust/` directory)
- Build system integration (Kbuild)
- Basic types and synchronization primitives
- Sample Rust module

### Linux 6.2 (February 2023) - Improvements

- Refined abstractions
- Better error handling
- Improved documentation
- Bug fixes and build system improvements

### Linux 6.3 (April 2023) - Continued Development

- Additional abstractions
- Better C interoperability
- Performance improvements
- Community feedback incorporation

### Linux 6.4+ (2023-2024) - Gradual Expansion

- **New subsystem support**: Platform drivers, GPIO, PHY
- **First real drivers**: Some ARM drivers being written in Rust
- **Tooling improvements**: Better integration with kernel tools
- **Community growth**: More contributors and reviewers

### Linux 6.7 (January 2024)

- Continued refinement of abstractions
- More driver examples
- Improved build system
- Better documentation

### Linux 6.8 (March 2024)

- Enhanced abstractions for device drivers
- Improved error handling patterns
- Better support for platform devices
- Rust minimum version updated to 1.73.0

### Linux 6.9 (May 2024)

- New Rust bindings for more subsystems
- Performance optimizations in Rust abstractions
- Additional driver examples and templates
- Improved integration with kernel testing frameworks

### Linux 6.10 (July 2024)

- Expanded filesystem support abstractions
- Enhanced networking subsystem bindings
- Better documentation for Rust API
- Community contributions increasing

### Linux 6.11 (September 2024)

- DRM (Direct Rendering Manager) subsystem support
- Rust NVMe driver improvements
- Additional safe wrappers for kernel APIs
- Growing adoption in ARM and RISC-V platforms

### Linux 6.12 (November 2024)

- PCI subsystem abstractions enhanced
- Scheduler and task abstractions improved
- Memory management abstractions refined
- Rust minimum version updated to 1.78.0

### Linux 6.13+ (2025-2026)

- Continued expansion of Rust in production drivers
- Mature abstractions for major subsystems
- Integration with eBPF and tracing
- Focus on performance and safety validation
- Growing number of out-of-tree Rust modules
- Enterprise adoption beginning to increase

## Key Technical Achievements

1. **Bindgen integration**: Automatic C binding generation
2. **Safe abstractions**: Memory-safe wrappers for kernel APIs
3. **Build system**: Seamless integration with Kbuild
4. **Documentation**: Rustdoc support for kernel APIs
5. **Testing**: Integration with kernel testing frameworks

## Community Reception Evolution

### Initial Skepticism (2020-2021)

- Concerns about "hype-driven development"
- Questions about long-term maintenance
- Worries about fragmenting the community
- Performance skepticism

### Growing Acceptance (2022-2023)

- Successful merge into mainline kernel
- Positive feedback from early adopters
- Real security benefits demonstrated
- Major companies showing interest

### Current State (2025-2026)

- Rust firmly established as a kernel language
- Multiple production drivers written in Rust
- Major subsystems have Rust abstractions
- Active development community with 100+ contributors
- Regular improvements in every kernel release
- Industry adoption growing in safety-critical systems
- Academic research validating safety benefits

## Timeline Summary

| Year | Milestone |
|------|-----------|
| 2019-2020 | Early explorations and discussions |
| 2021 | Project launch, RFC iterations |
| Dec 2022 | Linux 6.1 - Initial Rust support merged |
| 2023 | Continuous improvements across 6.2-6.7 |
| 2024 | Major expansion: 6.8-6.12, production drivers |
| 2025-2026 | Maturation, enterprise adoption, multiple subsystems |

## Key Figures and Contributors

- **Miguel Ojeda**: Project lead and primary maintainer
- **Linus Torvalds**: Approved experimental inclusion
- **Greg Kroah-Hartman**: Supportive of the initiative
- **Wedson Almeida Filho**: Core abstractions developer
- **Rust Community**: Ongoing language support

## Lessons Learned

1. **Patience required**: Kernel development moves deliberately
2. **Community engagement**: Regular communication essential
3. **Incremental approach**: Start small, prove value, expand
4. **Tooling matters**: Good tools accelerate adoption
5. **Education needed**: Teaching resources help onboarding

## References

- [LKML Rust-for-Linux discussions](https://lore.kernel.org/lkml/)
- [Rust-for-Linux GitHub repository](https://github.com/Rust-for-Linux/)
- [Linux kernel release notes](https://kernelnewbies.org/)
