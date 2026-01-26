# Code Snippets and Examples

This directory contains small code samples demonstrating Rust/C interoperability, kernel APIs, and other technical examples.

## Purpose

Use this space for:
- **Minimal reproducible examples** (MREs)
- **Rust/C interop demos**
- **Kernel API usage examples**
- **Build system fragments** (Kbuild, Makefiles)
- **Proof-of-concept code**
- **Learning exercises**

## What Belongs Here

✅ **Good fits**:
- Small, focused code examples (< 100 lines typically)
- Demonstration of specific concepts
- Template code for common patterns
- Working examples with explanations

❌ **Better elsewhere**:
- Full drivers → belongs in kernel tree
- Large projects → separate repository
- Production code → upstream submission
- Incomplete/broken code without educational value

## Organization

Suggested structure:
```
snippets/
├── ffi/                    # FFI and interop examples
│   ├── call_c_from_rust.rs
│   └── call_rust_from_c.c
├── kbuild/                 # Build system examples
│   ├── basic_module/
│   └── out_of_tree/
├── drivers/                # Driver code patterns
│   ├── minimal_driver.rs
│   └── platform_driver.rs
├── abstractions/           # Safe wrapper examples
│   └── safe_kmalloc.rs
└── tests/                  # Test examples
    └── kunit_example.rs
```

## Code Format

### Example Structure

Each snippet should include:

```rust
// SPDX-License-Identifier: GPL-2.0 OR MIT

//! Brief description of what this demonstrates
//!
//! This example shows how to [specific purpose].
//!
//! ## Build
//! ```bash
//! rustc --target=x86_64-unknown-none snippet.rs
//! ```
//!
//! ## Key Concepts
//! - Concept 1
//! - Concept 2

// Your code here
```

### Accompanying Documentation

Create a README.md in subdirectories:

```markdown
# Snippet Category

## Overview
What these examples demonstrate

## Examples

### example1.rs
- **Purpose**: What it does
- **Concepts**: What it teaches
- **Build**: How to compile/run
- **Notes**: Important details

### example2.c
- **Purpose**: ...
```

## Common Example Types

### 1. FFI (Foreign Function Interface)

```rust
// ffi/call_c_from_rust.rs

extern "C" {
    fn printk(fmt: *const u8, ...);
}

pub fn example() {
    unsafe {
        printk(b"Hello from Rust\n\0".as_ptr());
    }
}
```

### 2. Safe Abstractions

```rust
// abstractions/safe_kmalloc.rs

use core::ptr::NonNull;

pub struct KBox<T> {
    ptr: NonNull<T>,
}

impl<T> KBox<T> {
    pub fn new(value: T) -> Result<Self, Error> {
        // Safe allocation wrapper
    }
}
```

### 3. Kbuild Integration

```makefile
# kbuild/basic_module/Kbuild

obj-m := example.o
example-objs := example_rust.o
```

### 4. Driver Patterns

```rust
// drivers/minimal_driver.rs

use kernel::prelude::*;

module! {
    type: Example,
    name: "example",
    license: "GPL v2",
}

struct Example;

impl kernel::Module for Example {
    fn init() -> Result<Self> {
        pr_info!("Example loaded\n");
        Ok(Example)
    }
}
```

## Documentation Requirements

Each snippet should have:

1. **SPDX license**: Always include license identifier
2. **Description**: What it demonstrates
3. **Build instructions**: How to compile/use
4. **Comments**: Explain non-obvious parts
5. **Safety notes**: Document unsafe code

## Testing Snippets

If possible, snippets should:
- ✅ Compile without errors
- ✅ Follow Rust formatting (rustfmt)
- ✅ Pass clippy lints
- ✅ Include test cases if applicable

Run before committing:
```bash
rustfmt snippet.rs
cargo clippy -- snippet.rs
```

## Kbuild Examples

For build system examples, include:

```
kbuild_example/
├── README.md           # Build instructions
├── Kbuild             # Kernel build file
├── Makefile           # Optional: out-of-tree build
└── module.rs          # Rust source
```

**README.md** should explain:
- Purpose of the example
- How to build in-tree vs out-of-tree
- Required kernel configuration
- Expected output

## Contributing Snippets

When adding new snippets:

1. **Test first**: Ensure code compiles and runs
2. **Document well**: Clear comments and README
3. **Keep focused**: One concept per snippet
4. **Follow style**: Use rustfmt and kernel conventions
5. **License properly**: GPL-2.0 for kernel code

## Using Snippets

To use these examples:

1. **Read the documentation**: Understand what it does
2. **Check requirements**: Kernel version, config options
3. **Build and test**: Try compiling the example
4. **Adapt for your needs**: Modify as necessary
5. **Reference appropriately**: Cite source if publishing

## Example Index

Maintain an index file (`INDEX.md`) listing all snippets:

```markdown
# Snippet Index

## FFI Examples
- `ffi/call_c_from_rust.rs` - Calling C from Rust
- `ffi/call_rust_from_c.c` - Calling Rust from C

## Driver Examples
- `drivers/minimal_driver.rs` - Minimal kernel module
- `drivers/platform_driver.rs` - Platform device driver

## Build Examples
- `kbuild/basic/` - Simple in-tree module
- `kbuild/out_of_tree/` - Out-of-tree module build
```

## Resources

Related documentation:
- [Kernel Rust docs](https://docs.kernel.org/rust/)
- [Rust samples in kernel](https://github.com/Rust-for-Linux/linux/tree/rust-next/samples/rust)
- [Rust FFI guide](https://doc.rust-lang.org/nomicon/ffi.html)

## Questions?

- Check the main docs/ directory for broader context
- Ask on Zulip or mailing list
- Reference official kernel samples

---

**Happy coding!** Small examples lead to big understanding.
