# Toolchain & Dependencies

## Rust Version Policy

### Minimum Supported Rust Version (MSRV)

The kernel tracks a minimum Rust version:

- **Linux 6.1**: Rust 1.62.0
- **Linux 6.2**: Rust 1.62.0
- **Linux 6.3-6.4**: Rust 1.62.0+
- **Linux 6.5+**: Rust 1.68.0+
- **Linux 6.7+**: Rust 1.71.0+
- **Current mainline**: Check `Documentation/process/changes.rst`

### Version Upgrade Policy

The kernel upgrades the minimum Rust version when:
1. Significant language features become stable
2. Critical bug fixes are needed
3. Community consensus is reached
4. Sufficient time has passed (typically 6+ months after Rust release)

### Compiler Selection

**Recommended approach**: Use `rustup`

```bash
# Install specific version
rustup install 1.71.0

# Set as default
rustup default 1.71.0

# Or use override for kernel directory
cd linux
rustup override set 1.71.0
```

## Rustc and LLVM Constraints

### LLVM Version Compatibility

Rust uses LLVM as its backend, but kernel also uses LLVM for C compilation:

- **Requirement**: Compatible LLVM versions between Rust and kernel
- **Typical**: LLVM 14+ for recent kernels
- **Preferred**: Build kernel with `LLVM=1` to use same toolchain

### Compiler Flags

Key Rust compiler flags used in kernel:

```bash
# Target specification
--target=x86_64-unknown-none

# Optimization
-C opt-level=2

# Code model
-C code-model=kernel

# Panic strategy
-C panic=abort

# Debug info
-C debuginfo=2

# Relocation model
-C relocation-model=static
```

### Custom Target Specifications

The kernel uses custom target JSON files:

```json
{
  "arch": "x86_64",
  "data-layout": "e-m:e-...",
  "linker-flavor": "ld.lld",
  "target-endian": "little",
  "target-pointer-width": "64",
  "target-c-int-width": "32",
  "features": "+soft-float",
  "disable-redzone": true,
  "panic-strategy": "abort"
}
```

## Required Tools

### Core Requirements

1. **rustc**: Rust compiler
2. **rust-src**: Rust standard library source (for core/alloc)
3. **bindgen**: C binding generator
4. **rustfmt**: Code formatter
5. **clippy**: Linter (optional but recommended)

Install via rustup:
```bash
rustup component add rust-src
rustup component add rustfmt
rustup component add clippy
```

### Bindgen Requirements

Bindgen needs libclang:

```bash
# Ubuntu/Debian
sudo apt install libclang-dev

# Fedora
sudo dnf install clang-devel

# Arch
sudo pacman -S clang
```

Install bindgen:
```bash
cargo install bindgen-cli
```

### Additional Tools

- **LLVM tools**: `llvm-ar`, `llvm-nm`, `llvm-objcopy`
- **Make**: GNU Make 4.0+
- **Flex & Bison**: Parser generators
- **Perl**: For kernel scripts

## Cross-Compilation

### Adding Rust Targets

For cross-compilation, add target architecture:

```bash
# ARM64
rustup target add aarch64-unknown-none

# RISC-V
rustup target add riscv64gc-unknown-none-elf

# ARM (32-bit)
rustup target add armv7a-none-eabi
```

### Cross-Compilation Example

```bash
# Configure for ARM64
make ARCH=arm64 LLVM=1 CROSS_COMPILE=aarch64-linux-gnu- defconfig
make ARCH=arm64 LLVM=1 CROSS_COMPILE=aarch64-linux-gnu- menuconfig

# Enable Rust support
# General setup -> Rust support

# Build
make ARCH=arm64 LLVM=1 CROSS_COMPILE=aarch64-linux-gnu- -j$(nproc)
```

### Common Issues

- **Missing target**: Install with `rustup target add <target>`
- **Linker errors**: Ensure cross-compiler is in PATH
- **Bindgen issues**: Point to correct sysroot with `--sysroot`

## Building the Kernel with Rust

### Configuration Steps

1. **Get kernel source**:
   ```bash
   git clone https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git
   cd linux
   ```

2. **Configure kernel**:
   ```bash
   make LLVM=1 defconfig
   make LLVM=1 menuconfig
   ```

3. **Enable Rust support**:
   ```
   General setup --->
     [*] Rust support
   ```

4. **Build**:
   ```bash
   make LLVM=1 -j$(nproc)
   ```

### Verification

Check if Rust is properly configured:

```bash
make LLVM=1 rustavailable
```

This command checks:
- Rust compiler version
- Required components
- Bindgen availability
- LLVM compatibility

### Building Only Rust Code

To build just Rust components:

```bash
make LLVM=1 rust-analyzer
make LLVM=1 rustfmt
make LLVM=1 rustdoc
```

### Build Artifacts

Rust build generates:
- **rust/core.o**: Core library
- **rust/alloc.o**: Allocation library
- **rust/kernel.o**: Kernel abstractions
- **drivers/**/*.o**: Driver object files

## Testing Strategies

### QEMU Testing

Running kernel in QEMU:

```bash
# Build kernel with Rust
make LLVM=1 -j$(nproc)

# Create minimal root filesystem
# (see kernel documentation)

# Run in QEMU
qemu-system-x86_64 \
  -kernel arch/x86/boot/bzImage \
  -append "console=ttyS0 nokaslr" \
  -nographic \
  -initrd initramfs.cpio.gz
```

### KUnit Testing

KUnit is kernel's unit testing framework:

```bash
# Run KUnit tests
make LLVM=1 kunit

# Run specific test
./tools/testing/kunit/kunit.py run --arch=x86_64 rust
```

Example Rust KUnit test:

```rust
#[cfg(CONFIG_KUNIT)]
mod tests {
    use super::*;
    use kernel::kunit;

    #[kunit::test]
    fn test_basic_function() {
        let result = some_function();
        kunit::expect_eq!(result, expected);
    }
}
```

### Kselftest

Kernel selftests framework:

```bash
# Build selftests
make LLVM=1 -C tools/testing/selftests

# Run selftests
make LLVM=1 -C tools/testing/selftests run_tests
```

### Manual Testing

Load Rust module:

```bash
# Build module
make LLVM=1 M=samples/rust

# Load module
sudo insmod samples/rust/rust_minimal.ko

# Check dmesg
dmesg | tail

# Unload
sudo rmmod rust_minimal
```

## Development Environment Setup

### Recommended Setup

1. **Use recent Linux distribution**:
   - Ubuntu 22.04+
   - Fedora 37+
   - Arch Linux (rolling)

2. **Install build dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential flex bison libssl-dev \
     libelf-dev bc llvm clang lld libclang-dev

   # Fedora
   sudo dnf install gcc flex bison openssl-devel elfutils-libelf-devel \
     bc llvm clang lld clang-devel

   # Arch
   sudo pacman -S base-devel bc llvm clang lld
   ```

3. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup component add rust-src rustfmt clippy
   cargo install bindgen-cli
   ```

4. **Configure environment**:
   ```bash
   export LLVM=1
   export RUST_LIB_SRC=$(rustc --print sysroot)/lib/rustlib/src/rust/library
   ```

### IDE Support

**Rust-analyzer**:

```bash
# Generate rust-project.json
make LLVM=1 rust-analyzer

# Use with VS Code, Emacs, Vim, etc.
```

**Visual Studio Code**:
- Install "rust-analyzer" extension
- Point to generated `rust-project.json`

**CLion**:
- Rust plugin supports kernel development
- Configure with custom target

## Troubleshooting

### Common Errors

1. **"Rust compiler version too old"**:
   ```bash
   rustup update
   rustup override set 1.71.0  # or required version
   ```

2. **"bindgen not found"**:
   ```bash
   cargo install bindgen-cli
   # Ensure ~/.cargo/bin is in PATH
   ```

3. **"libclang not found"**:
   ```bash
   sudo apt install libclang-dev  # or equivalent
   ```

4. **"cannot find -lutil"**:
   - Missing libraries, install build dependencies

### Verification Commands

```bash
# Check Rust version
rustc --version

# Check components
rustup component list | grep installed

# Check bindgen
bindgen --version

# Check LLVM
clang --version

# Full kernel check
make LLVM=1 rustavailable
```

## Performance Considerations

### Build Times

- **Initial build**: Longer due to Rust compilation
- **Incremental builds**: Much faster with caching
- **Parallel builds**: Use `-j$(nproc)` for all cores

### Optimization Levels

```bash
# Debug build (faster compilation, slower runtime)
make LLVM=1

# Release build (slower compilation, faster runtime)
make LLVM=1 KCFLAGS="-O3"
```

### Caching

Enable `sccache` for faster rebuilds:

```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

## Resources

- [Rust-for-Linux Quick Start](https://rust-for-linux.com/quick-start)
- [Kernel Documentation: Rust](https://docs.kernel.org/rust/)
- [LLVM Documentation](https://llvm.org/docs/)
- [Rustup Book](https://rust-lang.github.io/rustup/)
