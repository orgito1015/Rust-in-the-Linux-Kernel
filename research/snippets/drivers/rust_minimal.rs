// SPDX-License-Identifier: GPL-2.0

//! Minimal Rust Kernel Module Example
//!
//! This demonstrates the simplest possible Rust kernel module that can be
//! built and loaded into a Linux kernel with Rust support enabled.
//!
//! ## Build
//!
//! To build this as an in-tree module:
//! 1. Place in `samples/rust/` directory
//! 2. Enable `CONFIG_RUST=y` and `CONFIG_SAMPLES_RUST=y`
//! 3. Run `make LLVM=1`
//!
//! To load:
//! ```bash
//! sudo insmod rust_minimal.ko
//! sudo rmmod rust_minimal
//! dmesg | tail
//! ```
//!
//! ## Key Concepts
//!
//! - `kernel::prelude::*` - Common kernel types and traits
//! - `module!` macro - Defines module metadata
//! - `Module` trait - Entry and exit points for the module
//! - `pr_info!` - Kernel logging macro (like printk)

use kernel::prelude::*;

/// Module metadata
module! {
    type: RustMinimal,
    name: "rust_minimal",
    author: "Research Project Example",
    description: "A minimal Rust kernel module example for learning",
    license: "GPL",
}

/// Module state structure
struct RustMinimal {
    // This could hold module-specific state
    // For this minimal example, it's empty
}

impl kernel::Module for RustMinimal {
    /// Called when module is loaded
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal module initialized successfully\n");
        pr_info!("This is a simple example of Rust in the kernel\n");
        
        Ok(RustMinimal {})
    }
}

impl Drop for RustMinimal {
    /// Called when module is unloaded
    fn drop(&mut self) {
        pr_info!("Rust minimal module is being removed\n");
    }
}

// Safety Notes:
// - This module contains no unsafe code
// - All kernel interactions are through safe abstractions
// - Memory safety is guaranteed by Rust's type system
// - No manual memory management required
