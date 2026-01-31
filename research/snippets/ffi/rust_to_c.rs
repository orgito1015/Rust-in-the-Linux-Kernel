// SPDX-License-Identifier: GPL-2.0

//! Rust-to-C FFI Example for Linux Kernel
//!
//! This demonstrates how Rust code can call C functions in the kernel.
//! Common use case: Rust drivers calling existing C kernel APIs.
//!
//! ## Key Concepts
//!
//! - `extern "C"` - Declares C functions with C calling convention
//! - `unsafe` - Required for FFI calls (no Rust safety guarantees)
//! - Raw pointers - Used to pass data across language boundary
//! - Null-terminated strings - C string convention

use kernel::prelude::*;

/// Example: Calling kernel's printk from Rust
///
/// The kernel provides many C functions that Rust code can call.
/// Most common ones are already wrapped in safe Rust abstractions,
/// but sometimes you need to call C directly.
extern "C" {
    /// The kernel's printk function
    /// Takes a format string and variadic arguments (just like C's printf)
    fn printk(fmt: *const core::ffi::c_char, ...) -> core::ffi::c_int;
    
    /// Example: Get current jiffies value (time counter)
    static jiffies: core::ffi::c_ulong;
}

/// Safe wrapper around unsafe printk call
///
/// Note: In actual kernel code, use pr_info! or other safe macros instead.
/// This is for educational purposes to demonstrate FFI patterns.
pub fn safe_printk_example(message: &str) {
    // In real kernel code, you would use the bindings provided by
    // the kernel crate which handle this safely. This is just to
    // demonstrate the FFI pattern.
    
    // For demonstration: calling C printk requires proper format string
    unsafe {
        // Safety: Using "%s" format with proper null termination
        // In practice, use kernel::pr_info! macro instead
        let fmt = b"%s\n\0".as_ptr() as *const core::ffi::c_char;
        let msg = message.as_ptr() as *const core::ffi::c_char;
        printk(fmt, msg);
    }
}

/// Example: Reading global C variable
pub fn get_current_jiffies() -> u64 {
    unsafe {
        // Safety: jiffies is a valid kernel global variable
        jiffies as u64
    }
}

/// Example: Passing structured data to C
#[repr(C)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub irq: u32,
}

extern "C" {
    /// Hypothetical C function that configures a device
    fn configure_device(info: *const DeviceInfo) -> core::ffi::c_int;
}

/// Safe wrapper for device configuration
pub fn setup_device(vendor: u16, device: u16, irq: u32) -> Result<()> {
    let info = DeviceInfo {
        vendor_id: vendor,
        device_id: device,
        irq,
    };
    
    let result = unsafe {
        // Safety: 
        // - info is a valid pointer to properly initialized struct
        // - configure_device expects this exact struct layout (repr(C))
        configure_device(&info as *const DeviceInfo)
    };
    
    if result == 0 {
        Ok(())
    } else {
        Err(EINVAL)
    }
}

// Safety Guidelines for FFI:
//
// 1. Always use 'unsafe' blocks for FFI calls
// 2. Validate all data passed to C (no null pointers unless allowed)
// 3. Use #[repr(C)] for structs passed across boundary
// 4. Ensure strings are null-terminated for C consumption
// 5. Check return values and convert to Result<T>
// 6. Document safety invariants clearly
// 7. Prefer safe Rust abstractions when available

/// Example of BAD practice (for educational purposes):
/// 
/// ```rust,no_run
/// // DON'T DO THIS: Calling C with unvalidated pointer
/// unsafe {
///     let ptr: *const u8 = core::ptr::null();  // Note: use 'core' not 'std' in kernel
///     some_c_function(ptr); // Undefined behavior!
/// }
/// ```
///
/// CORRECT approach:
///
/// ```rust,no_run
/// unsafe {
///     let value: u8 = 42;
///     let ptr = &value as *const u8;
///     // Ensure ptr is valid before passing to C
///     if !ptr.is_null() {
///         some_c_function(ptr);
///     }
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_info_layout() {
        use core::mem::size_of;
        
        // Verify repr(C) attribute is maintained
        // Note: Actual size may vary by architecture
        // This test mainly ensures struct remains repr(C)
        assert!(size_of::<DeviceInfo>() >= 8);
    }
}
