# Code Snippets Index

This directory contains practical code examples demonstrating Rust in the Linux kernel context.

## Available Examples

### Drivers

#### [rust_minimal.rs](./drivers/rust_minimal.rs)

**Purpose**: Simplest possible Rust kernel module

**Concepts Demonstrated**:

- Basic module structure with `module!` macro
- Module initialization and cleanup
- Kernel logging with `pr_info!`
- Module metadata declaration

**Build**: Place in `samples/rust/` and enable `CONFIG_RUST=y`

**Difficulty**: Beginner

---

### FFI (Foreign Function Interface)

#### [rust_to_c.rs](./ffi/rust_to_c.rs)

**Purpose**: Calling C functions from Rust code

**Concepts Demonstrated**:

- `extern "C"` declarations
- Unsafe FFI calls with safety documentation
- C-compatible struct layout with `#[repr(C)]`
- Converting between Rust and C string types
- Accessing C global variables
- Safe wrapper patterns around unsafe code

**Build**: Educational example, shows common patterns

**Difficulty**: Intermediate

---

## Using These Snippets

### For Learning

1. **Read the comments**: Each snippet is heavily documented
2. **Understand safety**: Pay attention to unsafe blocks and why they're needed
3. **Try modifications**: Experiment with the code to deepen understanding
4. **Build and test**: Actually compile and run the examples if possible

### For Reference

- Copy patterns into your own code
- Adapt to your specific use case
- Always maintain the safety invariants
- Keep documentation up to date

### For Contributing

When adding new snippets:

1. Follow the template in snippet README files
2. Include comprehensive comments
3. Document all unsafe code
4. Add entry to this index
5. Test that code compiles (if possible)

## Snippet Categories

### Current Coverage

- ✅ Basic modules
- ✅ FFI / C interop
- 📋 Platform drivers (planned)
- 📋 Character devices (planned)
- 📋 Synchronization primitives (planned)
- 📋 Memory allocation patterns (planned)
- 📋 Error handling (planned)

### Future Additions

We're looking to add examples for:

- Network device drivers
- Block device drivers  
- Filesystem operations
- Interrupt handlers
- DMA operations
- Device tree interaction
- GPIO and hardware control
- Power management
- Debugging techniques

## Conventions

All snippets follow these conventions:

- **License**: GPL-2.0 or dual GPL-2.0/MIT
- **Format**: rustfmt applied
- **Comments**: Comprehensive inline documentation
- **Safety**: All unsafe code justified
- **Testing**: Test cases when applicable

## Building Examples

### In-Tree Build

```bash
# Add snippet to kernel source
cp snippet.rs linux/samples/rust/

# Enable Rust support
cd linux
make LLVM=1 menuconfig  # Enable CONFIG_RUST

# Build
make LLVM=1 samples/rust/
```

### Out-of-Tree Build (if applicable)

Some snippets may include Kbuild/Makefile for out-of-tree compilation:

```bash
cd snippet_dir
make -C /lib/modules/$(uname -r)/build M=$PWD
```

## Learning Path

**Recommended order for beginners**:

1. Start with `rust_minimal.rs` - understand basic structure
2. Move to `rust_to_c.rs` - learn FFI basics
3. Try modifying examples
4. Build your own simple module
5. Graduate to more complex driver examples

## Resources

- [Kernel Rust Documentation](https://docs.kernel.org/rust/)
- [Rust for Linux Samples](https://github.com/Rust-for-Linux/linux/tree/rust/samples/rust)
- [Main Project Documentation](../../docs/)

## Contributing Snippets

See [main project CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

When contributing code examples:

- Ensure they compile (or clearly mark as pseudocode)
- Add comprehensive documentation
- Include safety justifications for unsafe code
- Add tests if applicable
- Update this index

---

**Last Updated**: January 2026

*Help us grow this collection! Contribute examples of patterns you find useful.*
