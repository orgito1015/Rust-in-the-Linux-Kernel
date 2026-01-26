# Development & Contribution

## Host Setup

### Distribution-Specific Setup

#### Ubuntu/Debian

```bash
# Install build dependencies
sudo apt update
sudo apt install -y \
  build-essential \
  flex \
  bison \
  libssl-dev \
  libelf-dev \
  bc \
  llvm \
  clang \
  lld \
  libclang-dev \
  linux-headers-$(uname -r)

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Rust components
rustup component add rust-src rustfmt clippy
cargo install bindgen-cli
```

#### Fedora

```bash
# Install build dependencies
sudo dnf install -y \
  gcc \
  flex \
  bison \
  openssl-devel \
  elfutils-libelf-devel \
  bc \
  llvm \
  clang \
  lld \
  clang-devel \
  kernel-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Rust components
rustup component add rust-src rustfmt clippy
cargo install bindgen-cli
```

#### Arch Linux

```bash
# Install build dependencies
sudo pacman -S \
  base-devel \
  bc \
  llvm \
  clang \
  lld \
  linux-headers

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Rust components
rustup component add rust-src rustfmt clippy
cargo install bindgen-cli
```

### Rustup Configuration

```bash
# Set stable Rust as default
rustup default stable

# Or use specific version for kernel development
rustup install 1.71.0
rustup default 1.71.0

# Verify installation
rustc --version
cargo --version
bindgen --version
```

### Environment Variables

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# Rust environment
export PATH="$HOME/.cargo/bin:$PATH"
export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/library"

# Kernel build with LLVM
export LLVM=1
```

## Kernel Source Setup

### Cloning the Kernel

```bash
# Official kernel repository
git clone https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git
cd linux

# Or use a stable version
git clone --branch v6.8 --depth 1 \
  https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git
cd linux
```

### Rust-for-Linux Development Tree

For latest Rust features:

```bash
git clone https://github.com/Rust-for-Linux/linux.git rust-linux
cd rust-linux
git checkout rust-next
```

## Build Workflows

### Basic Build

```bash
# Configure kernel
make LLVM=1 defconfig
make LLVM=1 menuconfig

# Enable: General setup -> Rust support

# Build
make LLVM=1 -j$(nproc)
```

### Incremental Build Workflow

```bash
# Initial full build
make LLVM=1 -j$(nproc)

# Make changes to Rust code
# Edit drivers/some_driver/rust_module.rs

# Incremental rebuild (much faster)
make LLVM=1 -j$(nproc)
```

### Build Specific Module

```bash
# Build only one module
make LLVM=1 M=drivers/staging/rust_example

# Clean and rebuild module
make LLVM=1 M=drivers/staging/rust_example clean
make LLVM=1 M=drivers/staging/rust_example
```

### Out-of-Tree Module Build

```bash
# Create module directory
mkdir my_rust_driver
cd my_rust_driver

# Create Kbuild file
cat > Kbuild << 'EOF'
obj-m := rust_driver.o
EOF

# Create Makefile
cat > Makefile << 'EOF'
KDIR ?= /lib/modules/$(shell uname -r)/build

default:
	$(MAKE) LLVM=1 -C $(KDIR) M=$(PWD)

clean:
	$(MAKE) LLVM=1 -C $(KDIR) M=$(PWD) clean
EOF

# Build
make
```

### Debugging Builds

```bash
# Verbose build
make LLVM=1 V=1 -j$(nproc)

# Check Rust availability
make LLVM=1 rustavailable

# Check configuration
make LLVM=1 rustversion
```

## Development Tools

### Rust-Analyzer

Generate project configuration for IDE support:

```bash
make LLVM=1 rust-analyzer
```

This creates `rust-project.json` for rust-analyzer.

**VS Code Setup**:
1. Install "rust-analyzer" extension
2. Open kernel directory
3. rust-analyzer automatically detects `rust-project.json`

### Formatting Code

```bash
# Format all Rust code
make LLVM=1 rustfmt

# Format specific file
rustfmt path/to/file.rs
```

### Linting with Clippy

```bash
# Run clippy
make LLVM=1 clippy

# Clippy is more strict than default compiler warnings
```

### Documentation Generation

```bash
# Generate documentation
make LLVM=1 rustdoc

# Documentation is placed in:
# Documentation/output/rust/
```

### Kernel Tags for Navigation

```bash
# Generate ctags/etags
make LLVM=1 tags
make LLVM=1 TAGS

# For rust-analyzer (better option)
make LLVM=1 rust-analyzer
```

## Testing Workflow

### Unit Tests with KUnit

```bash
# Run Rust KUnit tests
./tools/testing/kunit/kunit.py run --arch=x86_64 rust

# Run all tests
./tools/testing/kunit/kunit.py run --arch=x86_64
```

### QEMU Testing

```bash
# Build kernel
make LLVM=1 -j$(nproc)

# Create initramfs (example with busybox)
# See kernel documentation for details

# Run in QEMU
qemu-system-x86_64 \
  -kernel arch/x86/boot/bzImage \
  -initrd initramfs.cpio.gz \
  -append "console=ttyS0 nokaslr" \
  -nographic \
  -enable-kvm \
  -m 2G
```

### Real Hardware Testing

```bash
# Build and install
make LLVM=1 -j$(nproc)
sudo make modules_install
sudo make install

# Update bootloader
sudo update-grub  # Debian/Ubuntu
sudo grub2-mkconfig -o /boot/grub2/grub.cfg  # Fedora

# Reboot into new kernel
sudo reboot
```

## Contribution Process

### Understanding the Kernel Workflow

1. **Mailing list based**: Patches sent via email to LKML
2. **No pull requests**: Unlike GitHub workflow
3. **Sign-offs required**: Developer Certificate of Origin
4. **Review process**: Public discussion on mailing list
5. **Maintainer tree**: Patches merged through subsystem maintainers

### Preparing Your Contribution

#### 1. Configure Git

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

#### 2. Create a Branch

```bash
git checkout -b my-rust-feature
```

#### 3. Make Changes

```bash
# Edit Rust files
vim drivers/staging/my_driver/rust_code.rs

# Follow kernel coding style
# Follow Rust style (rustfmt)
```

#### 4. Test Thoroughly

```bash
# Build
make LLVM=1 -j$(nproc)

# Run tests
./tools/testing/kunit/kunit.py run --arch=x86_64

# Test on real hardware if possible
```

### Commit Messages

Follow kernel commit message format:

```
subsystem: Brief summary (up to 50 chars)

Longer explanation of the change, wrapped at 72 characters.
Explain what was changed and why, not how.

If fixing a bug, reference the commit that introduced it:
Fixes: 1234567890ab ("commit title")

Add any relevant tags:
Reported-by: Name <email@example.com>
Tested-by: Name <email@example.com>
Reviewed-by: Name <email@example.com>

Signed-off-by: Your Name <your.email@example.com>
```

Example:

```
rust: Add support for GPIO interrupts

This patch adds interrupt support to the GPIO abstractions in Rust.
It provides safe wrappers around the kernel's IRQ handling APIs,
allowing Rust drivers to register interrupt handlers.

The implementation ensures that interrupt handlers meet Rust's
safety requirements while maintaining compatibility with the
kernel's existing IRQ infrastructure.

Signed-off-by: Jane Developer <jane@example.com>
```

### Sign-Offs

The sign-off certifies you have the right to submit the code:

```bash
# Add sign-off automatically
git commit -s

# Or add to existing commit
git commit --amend -s
```

### Creating Patches

```bash
# Generate patch for last commit
git format-patch -1

# Generate patches for series
git format-patch -3  # Last 3 commits

# Add cover letter for series
git format-patch --cover-letter -3
```

### Sending Patches

#### Setup git send-email

```bash
# Install git-email
sudo apt install git-email  # Ubuntu/Debian
sudo dnf install git-email  # Fedora

# Configure
git config --global sendemail.smtpserver smtp.gmail.com
git config --global sendemail.smtpserverport 587
git config --global sendemail.smtpencryption tls
git config --global sendemail.smtpuser your.email@gmail.com
```

#### Send Patch

```bash
# Get maintainer list
./scripts/get_maintainer.pl 0001-your-patch.patch

# Send to maintainers and lists
git send-email \
  --to=maintainer@example.com \
  --cc=rust-for-linux@vger.kernel.org \
  --cc=linux-kernel@vger.kernel.org \
  0001-your-patch.patch
```

### Style and Conventions

#### Rust Code Style

- **Use rustfmt**: `make LLVM=1 rustfmt`
- **Follow Rust conventions**: Snake_case for functions, CamelCase for types
- **Document public APIs**: Use doc comments (`///`)
- **Keep lines under 100 characters** (kernel standard, not Rust's 80)

#### Kernel Rust Conventions

- **Use kernel macros**: `pr_info!()`, `pr_err!()`, etc.
- **Error handling**: Return `Result<T>` or `Result<T, Error>`
- **SAFETY comments**: Document all unsafe code
- **Module metadata**: Always include license, author, description

Example:

```rust
// SPDX-License-Identifier: GPL-2.0

//! My Rust driver
//!
//! This driver provides support for XYZ device.

use kernel::prelude::*;

module! {
    type: MyDriver,
    name: "my_driver",
    author: "Your Name",
    description: "XYZ device driver",
    license: "GPL v2",
}

struct MyDriver;

impl kernel::Module for MyDriver {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("My driver initialized\n");
        Ok(MyDriver)
    }
}
```

### Review Process

1. **Submit patch**: Send to mailing list
2. **Wait for review**: Can take days to weeks
3. **Address feedback**: Make changes, send v2
4. **Iterate**: Continue until maintainer accepts
5. **Merged**: Patch goes into maintainer tree, then mainline

### Tips for Success

- **Start small**: Simple fixes or additions
- **Read existing code**: Learn from current Rust modules
- **Join community**: Subscribe to rust-for-linux@vger.kernel.org
- **Be patient**: Kernel development takes time
- **Test thoroughly**: Hardware testing is highly valued
- **Document well**: Clear documentation helps review

## Approaching Rust-for-Linux Maintainers

### Key Contacts

- **Miguel Ojeda**: Rust maintainer (ojeda@kernel.org)
- **rust-for-linux@vger.kernel.org**: Main mailing list
- **linux-kernel@vger.kernel.org**: General kernel list

### Communication Guidelines

1. **Be respectful**: Maintainers are volunteers
2. **Be clear**: Explain your changes well
3. **Be patient**: Reviews take time
4. **Accept feedback**: Learn from critique
5. **Follow up**: Don't abandon your patches

### Mailing List Etiquette

- **Plain text only**: No HTML email
- **No top-posting**: Reply inline or at bottom
- **Trim quotes**: Don't quote entire messages
- **Stay on topic**: Keep discussions focused
- **CC appropriately**: Use get_maintainer.pl

## Resources for Contributors

- [Kernel Documentation](https://docs.kernel.org/)
- [Rust-for-Linux Docs](https://rust-for-linux.github.io/)
- [Submitting Patches](https://www.kernel.org/doc/html/latest/process/submitting-patches.html)
- [Coding Style](https://www.kernel.org/doc/html/latest/process/coding-style.html)
- [Email Clients](https://www.kernel.org/doc/html/latest/process/email-clients.html)

## Getting Help

- **IRC**: #rust-for-linux on OFTC
- **Mailing list**: rust-for-linux@vger.kernel.org
- **Zulip**: rust-for-linux.zulipchat.com
- **GitHub Discussions**: Limited use, prefer mailing list
