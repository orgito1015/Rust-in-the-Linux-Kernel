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

#### [dma_operations.rs](./drivers/dma_operations.rs)

**Purpose**: DMA and hardware interaction safety

**Concepts Demonstrated**:

- Coherent (consistent) DMA allocation with RAII cleanup
- Streaming DMA with explicit cache synchronisation
- DMA direction encoded in the type system
- Memory-mapped I/O (MMIO) via volatile reads/writes
- DMA descriptor ring layout with `#[repr(C)]`
- IOMMU and DMA mask concepts

**Build**: Educational example; requires kernel build environment for real DMA calls

**Difficulty**: Advanced

---

#### [interrupt_handler.rs](./drivers/interrupt_handler.rs)

**Purpose**: Interrupt handler patterns in Rust

**Concepts Demonstrated**:

- IRQ return values (`IRQ_HANDLED`, `IRQ_NONE`, `IRQ_WAKE_THREAD`)
- Simple top-half handler with atomic device state
- Top-half / bottom-half split using a workqueue
- Threaded interrupt handler (`request_threaded_irq`)
- Safe per-device data access from IRQ context
- IRQ context constraints (no sleeping, no blocking allocations)

**Build**: Educational example; patterns apply directly to real kernel drivers

**Difficulty**: Advanced

---

### Concurrency

#### [lock_free.rs](./concurrency/lock_free.rs)

**Purpose**: Lock-free data structure implementations

**Concepts Demonstrated**:

- Atomic counter with explicit memory ordering (`Relaxed`, `Acquire`, `Release`)
- Treiber lock-free stack using compare-and-swap
- Atomic reference counting (Arc-like RAII wrapper)
- RCU (Read-Copy-Update) reader/writer pattern
- Memory ordering cheat-sheet

**Build**: Compilable with standard Rust toolchain for tests; kernel patterns for in-tree use

**Difficulty**: Advanced

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

### Memory

#### [memory_allocator.rs](./memory/memory_allocator.rs)

**Purpose**: Memory allocator integration patterns

**Concepts Demonstrated**:

- Heap allocation with `Box<T>` (maps to `kmalloc`/`kfree`)
- Dynamic arrays with `Vec<T>` using `try_push`
- GFP flags: `GFP_KERNEL`, `GFP_ATOMIC`, `GFP_DMA`
- Custom RAII wrapper over raw kernel memory
- Slab cache (`kmem_cache`) pattern for fixed-size objects
- Vmalloc for large, non-physically-contiguous allocations
- Allocator selection guide

**Build**: Educational example; requires kernel build for real allocator calls

**Difficulty**: Intermediate

---

### Error Handling

#### [error_propagation.rs](./error/error_propagation.rs)

**Purpose**: Error propagation mechanisms

**Concepts Demonstrated**:

- Kernel `Error` type and common errno values
- The `?` operator for concise error propagation
- Converting C integer return codes to `Result`
- `map_err` for error context enrichment
- `Option` ↔ `Result` conversions (`ok_or`, `ok`)
- RAII for automatic cleanup on error (replaces C `goto cleanup`)
- Error handling across FFI boundaries
- Collecting results from iterators

**Build**: Compilable with standard Rust toolchain for tests

**Difficulty**: Beginner–Intermediate

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
- ✅ Memory allocation patterns
- ✅ Lock-free data structures
- ✅ DMA and hardware interaction
- ✅ Interrupt handlers
- ✅ Error handling
- 📋 Platform drivers (planned)
- 📋 Character devices (planned)
- 📋 Synchronization primitives (planned)

### Future Additions (Planned)

We're looking to add examples for:

- 📋 Network device drivers
- 📋 Block device drivers
- 📋 Filesystem operations
- 📋 Device tree interaction
- 📋 GPIO and hardware control
- 📋 Power management
- 📋 Debugging techniques

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
3. Read `error_propagation.rs` - master Rust error handling
4. Study `memory_allocator.rs` - understand kernel memory management
5. Explore `interrupt_handler.rs` - learn IRQ handler patterns
6. Dive into `lock_free.rs` - advanced concurrency without locks
7. Graduate to `dma_operations.rs` - hardware interaction patterns
8. Build your own simple module

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
