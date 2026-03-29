// SPDX-License-Identifier: GPL-2.0

//! Error Propagation Mechanisms in Rust Kernel Code
//!
//! Error handling is one of the areas where Rust's design most improves over C.
//! The Linux kernel's C convention (negative errno integers, NULL pointers,
//! global `errno`) is replaced with Rust's `Result<T, Error>` type, which:
//!
//! - Makes error paths **explicit** and **statically checked**
//! - Prevents "forgot to check the return value" bugs
//! - Enables the `?` operator for ergonomic propagation
//! - Integrates with the kernel's existing errno values
//!
//! ## Topics Covered
//!
//! 1. The kernel `Error` type and common errno values
//! 2. The `?` operator for concise propagation
//! 3. Converting from C integer return codes
//! 4. Custom error context with `map_err`
//! 5. Error handling in module init / cleanup
//! 6. Working with `Option` alongside `Result`
//! 7. Error handling across FFI boundaries
//!
//! ## Build
//!
//! Place in `samples/rust/` and enable `CONFIG_RUST=y`.
//!
//! ## Difficulty
//!
//! Beginner–Intermediate

use kernel::prelude::*;

// ---------------------------------------------------------------------------
// 1. Basic Result usage with kernel errno values
// ---------------------------------------------------------------------------

/// The kernel crate re-exports `Error` which wraps a Linux errno integer.
/// Common values (all available as constants in the kernel crate):
///
/// | Constant | errno | Meaning                           |
/// |----------|-------|-----------------------------------|
/// | ENOMEM   | 12    | Out of memory                     |
/// | EINVAL   | 22    | Invalid argument                  |
/// | ENODEV   | 19    | No such device                    |
/// | EBUSY    | 16    | Device or resource busy           |
/// | ENOTSUPP | 524   | Operation not supported           |
/// | ENOSYS   | 38    | Function not implemented          |
/// | EIO      | 5     | I/O error                         |
/// | ETIMEDOUT| 110   | Connection timed out              |
/// | EACCES   | 13    | Permission denied                 |
///
/// Returning `Ok(value)` signals success; `Err(errno)` signals failure.
pub fn read_device_register(valid: bool) -> Result<u32> {
    if valid {
        Ok(0xDEAD_BEEF)
    } else {
        Err(EIO)
    }
}

// ---------------------------------------------------------------------------
// 2. The `?` operator — propagate errors without boilerplate
// ---------------------------------------------------------------------------

/// Demonstrates chained `?` calls.
///
/// Without `?`, every call would require an explicit `match` or `if let Err`.
/// With `?`, the error is returned immediately to the caller on failure.
pub fn initialise_device() -> Result<()> {
    // Each `?` returns early with the error if any step fails.
    let _reg0 = read_device_register(true)?;
    let _reg1 = read_device_register(true)?;

    pr_info!("Device initialised successfully\n");
    Ok(())
}

/// A more realistic driver init sequence.
pub fn driver_init_sequence() -> Result<()> {
    // Step 1: Validate hardware presence.
    let _id = read_device_id()?;

    // Step 2: Allocate driver state.
    let _state = allocate_driver_state()?;

    // Step 3: Configure hardware.
    configure_hardware()?;

    // Step 4: Register with subsystem.
    register_with_subsystem()?;

    Ok(())
}

fn read_device_id() -> Result<u32> {
    // Simulated: returns a valid device ID.
    Ok(0x1234_5678)
}

fn allocate_driver_state() -> Result<Box<[u8; 64]>> {
    Box::try_new([0u8; 64]).map_err(|_| ENOMEM)
}

fn configure_hardware() -> Result<()> {
    // Simulated hardware configuration.
    pr_info!("Hardware configured\n");
    Ok(())
}

fn register_with_subsystem() -> Result<()> {
    // Simulated subsystem registration.
    pr_info!("Registered with subsystem\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Converting C integer return codes to Result
// ---------------------------------------------------------------------------

/// Wrap a C function that returns 0 on success and a negative errno on error.
///
/// This pattern is ubiquitous when calling `bindings::*` functions.
pub fn call_c_function_safely() -> Result<()> {
    // Simulate a C function call returning 0 (success).
    let ret: i32 = 0; // In real code: unsafe { bindings::some_c_function(...) }

    if ret == 0 {
        Ok(())
    } else {
        // Convert negative errno to kernel Error.
        // The kernel crate provides `Error::from_errno` / `to_result`.
        Err(EINVAL) // In real code: Err(Error::from_errno(-ret))
    }
}

/// Helper that converts a C-style `int` return to `Result<()>`.
///
/// Maps 0 → `Ok(())` and negative values → `Err(errno)`.
pub fn to_result(ret: i32) -> Result<()> {
    if ret >= 0 {
        Ok(())
    } else {
        // In real kernel code: kernel::error::to_result(ret)
        Err(EINVAL)
    }
}

/// Helper for C functions that return a pointer (NULL on error).
pub fn ptr_to_result<T>(ptr: *mut T) -> Result<core::ptr::NonNull<T>> {
    core::ptr::NonNull::new(ptr).ok_or(ENOMEM)
}

// ---------------------------------------------------------------------------
// 4. `map_err` — enrich errors with context
// ---------------------------------------------------------------------------

/// Wraps errors with additional context using `map_err`.
///
/// Since the kernel's `Error` type is a single errno integer, enrichment
/// typically means logging before propagating, or choosing a more appropriate
/// errno for the caller.
pub fn open_device_file(name: &str) -> Result<u32> {
    // Simulate failing to find the device.
    find_device(name).map_err(|e| {
        pr_err!("Failed to open device '{}': {:?}\n", name, e);
        ENODEV // Return ENODEV regardless of the original error
    })
}

fn find_device(_name: &str) -> Result<u32> {
    // Simulate device not found.
    Err(ENODEV)
}

// ---------------------------------------------------------------------------
// 5. `Option` ↔ `Result` conversions
// ---------------------------------------------------------------------------

/// Shows conversions between `Option<T>` and `Result<T, Error>`.
///
/// `Option::ok_or(err)` converts `None` → `Err(err)`.
/// `Result::ok()` converts `Err(_)` → `None`.
pub fn find_and_validate(haystack: &[u32], needle: u32) -> Result<usize> {
    // `position` returns Option<usize>.
    // `.ok_or(ENOENT)` converts None → Err(ENOENT).
    haystack
        .iter()
        .position(|&x| x == needle)
        .ok_or(ENODEV)
}

/// Uses `Option` for nullable pointers and converts to `Result` at the boundary.
pub fn get_optional_resource() -> Result<u32> {
    let maybe_resource: Option<u32> = Some(42);
    maybe_resource.ok_or(ENODEV)
}

// ---------------------------------------------------------------------------
// 6. Error handling across the module init / cleanup boundary
// ---------------------------------------------------------------------------

/// Demonstrates resource cleanup on partial initialisation failure.
///
/// In C, this is handled with `goto cleanup` labels. In Rust, RAII handles
/// cleanup automatically: if `?` returns an error, any values already
/// constructed are dropped in reverse order.
pub struct DriverResources {
    _buffer: Box<[u8; 128]>,
    _device_id: u32,
}

impl DriverResources {
    /// Allocate all driver resources. On failure, already-allocated resources
    /// are freed automatically by their `Drop` impls.
    pub fn init() -> Result<Self> {
        // Step 1: allocate DMA buffer.
        let buffer = Box::try_new([0u8; 128]).map_err(|_| ENOMEM)?;

        // Step 2: probe device ID.
        let device_id = read_device_id()?;

        // Step 3: further initialisation (may fail).
        configure_hardware()?;

        Ok(Self {
            _buffer: buffer,
            _device_id: device_id,
        })
        // If configure_hardware() returns Err, `buffer` is already freed here
        // because it goes out of scope. No goto/cleanup labels needed.
    }
}

// ---------------------------------------------------------------------------
// 7. Error handling at FFI boundaries
// ---------------------------------------------------------------------------

/// Pattern for calling an `unsafe` C function and wrapping its return.
///
/// # Safety
///
/// The caller must ensure that `data_ptr` is a valid pointer to at least
/// `len` bytes of readable memory.
pub unsafe fn safe_ffi_wrapper(data_ptr: *const u8, len: usize) -> Result<u32> {
    if data_ptr.is_null() || len == 0 {
        return Err(EINVAL);
    }

    // Call a hypothetical C function:
    //   extern "C" { fn process_data(ptr: *const u8, len: usize) -> i32; }
    // let ret = unsafe { process_data(data_ptr, len) };
    let ret: i32 = 0; // Simulated return value

    to_result(ret)?;
    Ok(len as u32)
}

// ---------------------------------------------------------------------------
// 8. Propagating errors from closures
// ---------------------------------------------------------------------------

/// Collects results from a set of operations; first failure short-circuits.
pub fn batch_operations(items: &[u32]) -> Result<Vec<u32>> {
    items
        .iter()
        .map(|&item| process_item(item))
        .collect()
}

fn process_item(item: u32) -> Result<u32> {
    if item == 0 {
        Err(EINVAL)
    } else {
        Ok(item * 2)
    }
}

// ---------------------------------------------------------------------------
// Error propagation cheat-sheet (comments)
// ---------------------------------------------------------------------------
//
//  Pattern                    | Use case
//  ---------------------------|------------------------------------------------
//  `expr?`                    | Propagate error immediately to caller
//  `.map_err(|e| new_err)`    | Transform error type or add context
//  `.ok_or(err)`              | Convert Option::None → Err(err)
//  `.unwrap_or_default()`     | Use default value on error (rarely appropriate)
//  `match result { ... }`     | Handle specific error variants differently
//  `if let Err(e) = result`   | Log-and-continue pattern
//
//  Never use `.unwrap()` in production kernel code — it panics, which is a BUG().

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_device_register_ok() {
        assert!(read_device_register(true).is_ok());
        assert_eq!(read_device_register(true).unwrap(), 0xDEAD_BEEF);
    }

    #[test]
    fn test_read_device_register_err() {
        assert!(read_device_register(false).is_err());
    }

    #[test]
    fn test_find_and_validate_found() {
        let data = [1u32, 2, 3, 4, 5];
        assert_eq!(find_and_validate(&data, 3).unwrap(), 2);
    }

    #[test]
    fn test_find_and_validate_not_found() {
        let data = [1u32, 2, 3];
        assert!(find_and_validate(&data, 99).is_err());
    }

    #[test]
    fn test_ptr_to_result_null() {
        let null: *mut u32 = core::ptr::null_mut();
        assert!(ptr_to_result(null).is_err());
    }

    #[test]
    fn test_ptr_to_result_valid() {
        let mut val: u32 = 42;
        let ptr = &mut val as *mut u32;
        assert!(ptr_to_result(ptr).is_ok());
    }

    #[test]
    fn test_to_result() {
        assert!(to_result(0).is_ok());
        assert!(to_result(-1).is_err());
        assert!(to_result(1).is_ok());
    }

    #[test]
    fn test_batch_operations_ok() {
        let items = [1u32, 2, 3];
        let result = batch_operations(&items).unwrap();
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_batch_operations_err() {
        let items = [1u32, 0, 3];
        assert!(batch_operations(&items).is_err());
    }
}
