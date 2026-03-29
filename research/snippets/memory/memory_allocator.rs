// SPDX-License-Identifier: GPL-2.0

//! Memory Allocator Integration Patterns in the Linux Kernel
//!
//! This module demonstrates how Rust code integrates with the Linux kernel's
//! memory allocation subsystem. The kernel provides several allocators:
//!
//! - **SLUB/SLAB**: The primary kernel object allocator (`kmalloc`/`kfree`)
//! - **Vmalloc**: Virtually contiguous, not physically contiguous (`vmalloc`/`vfree`)
//! - **Page allocator**: Direct page allocation (`alloc_pages`/`free_pages`)
//! - **Percpu allocator**: Per-CPU variable storage
//!
//! ## Key Concepts
//!
//! - GFP flags control allocation behavior (e.g., `GFP_KERNEL`, `GFP_ATOMIC`)
//! - Rust's `Box<T>` maps to `kmalloc`/`kfree` via the global allocator
//! - `KBox<T>` (or `Box<T, KernelAllocator>`) carries GFP flags in the type system
//! - RAII wrappers ensure memory is freed when values are dropped
//! - `GFP_ATOMIC` must be used in interrupt context (no sleeping allowed)
//!
//! ## Build
//!
//! Place in `samples/rust/` and enable `CONFIG_RUST=y`.
//!
//! ## Difficulty
//!
//! Intermediate

use kernel::prelude::*;

// ---------------------------------------------------------------------------
// 1. Basic heap allocation with the kernel allocator
// ---------------------------------------------------------------------------

/// Wraps a heap-allocated kernel object.
///
/// Uses `Box<T>` backed by the kernel's `kmalloc`/`kfree` implementation.
/// The allocation happens with `GFP_KERNEL`, which is the standard flag for
/// allocations that can sleep (i.e., process context only).
pub fn basic_heap_allocation() -> Result<Box<u32>> {
    // Box::new calls the global kernel allocator under the hood.
    // If allocation fails, the kernel returns ENOMEM.
    let value = Box::try_new(42u32)?;
    pr_info!("Allocated value on kernel heap: {}\n", *value);
    // `value` is freed automatically when it goes out of scope.
    Ok(value)
}

// ---------------------------------------------------------------------------
// 2. Vec-based dynamic arrays
// ---------------------------------------------------------------------------

/// Demonstrates dynamic array allocation.
///
/// `Vec<T>` in the kernel uses `krealloc` under the hood. Always use
/// `try_push` / `Vec::try_with_capacity` so allocation failures are
/// propagated as `ENOMEM` rather than panicking.
pub fn dynamic_array_example() -> Result<Vec<u8>> {
    let mut buffer = Vec::try_with_capacity(64)?;

    for i in 0u8..64 {
        buffer.try_push(i)?;
    }

    pr_info!("Allocated Vec of {} bytes\n", buffer.len());
    Ok(buffer)
}

// ---------------------------------------------------------------------------
// 3. GFP flags and allocation contexts
// ---------------------------------------------------------------------------

/// Allocation flags guide the allocator on *how* to allocate memory.
///
/// # GFP_KERNEL
/// Standard flag. The allocator may sleep/reclaim memory. Only usable in
/// process context.
///
/// # GFP_ATOMIC
/// Non-blocking. Must be used in:
/// - Interrupt handlers
/// - Softirqs / tasklets
/// - Any context where sleeping is forbidden
/// Returns `ENOMEM` immediately if no free pages are available.
///
/// # GFP_NOWAIT
/// Like `GFP_ATOMIC` but with weaker reclaim pressure. Use when you can
/// tolerate failure and will retry later.
///
/// # GFP_DMA / GFP_DMA32
/// Restrict allocation to DMA-accessible memory zones. Required for
/// hardware that cannot address all of physical memory.
pub struct AllocationContext;

impl AllocationContext {
    /// Allocate in process context (can sleep).
    pub fn allocate_process_context() -> Result<Box<[u8; 128]>> {
        // Box::try_new uses GFP_KERNEL by default in the kernel allocator.
        // GFP_KERNEL: normal kernel allocation, may sleep/reclaim memory.
        // This flag is baked into the kernel crate's global allocator;
        // callers in process context should always use Box/Vec for this reason.
        let buf = Box::try_new([0u8; 128])?;
        Ok(buf)
    }

    /// Allocate in interrupt context (must NOT sleep).
    ///
    /// In real code, pass `GFP_ATOMIC` to the allocator.
    /// Illustrated here as a pattern; actual kernel crate API may differ.
    pub fn allocate_interrupt_context() -> Result<Box<[u8; 64]>> {
        // GFP_ATOMIC: allocation may fail rather than sleeping
        // SAFETY: called from a context where sleeping is forbidden
        let buf = Box::try_new([0u8; 64])?;
        pr_info!("Atomic allocation succeeded\n");
        Ok(buf)
    }
}

// ---------------------------------------------------------------------------
// 4. Custom RAII wrapper over raw kernel memory
// ---------------------------------------------------------------------------

/// A RAII wrapper that holds a raw pointer to kernel-allocated memory.
///
/// Demonstrates the pattern of wrapping `kmalloc`/`kfree` when the safe
/// kernel-crate wrappers are not yet available for a particular API.
pub struct KernelBuffer {
    ptr: core::ptr::NonNull<u8>,
    len: usize,
}

// SAFETY: Kernel memory is not thread-local; sharing across threads is safe
// as long as no data races occur (enforced by &mut access or synchronization).
unsafe impl Send for KernelBuffer {}
unsafe impl Sync for KernelBuffer {}

impl KernelBuffer {
    /// Allocate `len` bytes using the kernel allocator.
    ///
    /// # Errors
    ///
    /// Returns `ENOMEM` if the allocation fails.
    pub fn new(len: usize) -> Result<Self> {
        // In real code: bindings::kmalloc(len, bindings::GFP_KERNEL)
        // For illustration, we use Box<[u8]> as a stand-in.
        let boxed = Box::try_new_slice(len, 0u8)?;
        // Leak the Box and keep the raw pointer.
        let ptr = Box::into_raw(boxed) as *mut u8;
        // SAFETY: Box::into_raw always returns a non-null pointer.
        let ptr = unsafe { core::ptr::NonNull::new_unchecked(ptr) };
        Ok(Self { ptr, len })
    }

    /// Write a byte at `offset`.
    ///
    /// # Panics
    ///
    /// Panics if `offset >= self.len`.
    pub fn write_byte(&mut self, offset: usize, value: u8) {
        assert!(offset < self.len, "offset out of bounds");
        // SAFETY: offset is bounds-checked above; ptr is valid for `self.len` bytes.
        unsafe { self.ptr.as_ptr().add(offset).write(value) };
    }

    /// Read a byte at `offset`.
    ///
    /// # Panics
    ///
    /// Panics if `offset >= self.len`.
    pub fn read_byte(&self, offset: usize) -> u8 {
        assert!(offset < self.len, "offset out of bounds");
        // SAFETY: offset is bounds-checked above; ptr is valid for `self.len` bytes.
        unsafe { self.ptr.as_ptr().add(offset).read() }
    }
}

impl Drop for KernelBuffer {
    fn drop(&mut self) {
        // Reconstruct the Box so it deallocates properly.
        // SAFETY: ptr was created from Box::into_raw with element count `self.len`.
        unsafe {
            let slice_ptr =
                core::ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len);
            drop(Box::from_raw(slice_ptr));
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Memory pools (slab caches)
// ---------------------------------------------------------------------------

/// Pattern for a fixed-size slab cache (kmem_cache).
///
/// `kmem_cache` is the preferred allocator when many objects of the same
/// size are created and destroyed frequently. It reduces fragmentation and
/// improves allocation speed through object reuse.
///
/// The actual `kmem_cache_create` / `kmem_cache_destroy` calls live in the
/// `bindings` crate; this struct shows the RAII ownership pattern.
pub struct SlabCache {
    // In real code this would be: ptr: *mut bindings::kmem_cache,
    // Using a placeholder type here for illustration.
    _name: &'static kernel::str::CStr,
    object_size: usize,
}

impl SlabCache {
    /// Create a named slab cache for objects of `object_size` bytes.
    pub fn new(name: &'static kernel::str::CStr, object_size: usize) -> Result<Self> {
        pr_info!("Creating slab cache for {} byte objects\n", object_size);
        // Real code:
        //   let ptr = unsafe {
        //       bindings::kmem_cache_create(name.as_char_ptr(), object_size, ...)
        //   };
        //   if ptr.is_null() { return Err(ENOMEM); }
        Ok(Self { _name: name, object_size })
    }

    /// Allocate one object from the cache.
    pub fn alloc(&self) -> Result<*mut u8> {
        pr_info!("Allocating {} byte object from slab cache\n", self.object_size);
        // Real code:
        //   let obj = unsafe { bindings::kmem_cache_alloc(self.ptr, bindings::GFP_KERNEL) };
        //   if obj.is_null() { return Err(ENOMEM); }
        //   Ok(obj as *mut u8)
        Err(ENOSYS) // Stub: not implemented outside real kernel build
    }
}

impl Drop for SlabCache {
    fn drop(&mut self) {
        pr_info!("Destroying slab cache\n");
        // Real code: unsafe { bindings::kmem_cache_destroy(self.ptr); }
    }
}

// ---------------------------------------------------------------------------
// 6. Vmalloc for large, virtually-contiguous allocations
// ---------------------------------------------------------------------------

/// When you need a large buffer that does not need to be *physically*
/// contiguous, `vmalloc` maps multiple non-contiguous pages into a single
/// virtually-contiguous range.
///
/// Use cases:
/// - Large driver firmware buffers
/// - Kernel module `.text`/`.data` sections
/// - Buffers too large for `kmalloc` (> 4 MiB typically)
///
/// **Note**: vmalloc memory cannot be used for DMA without an IOMMU.
pub struct VmallocBuffer {
    // ptr: *mut c_void — in real kernel code via bindings::vmalloc
    len: usize,
}

impl VmallocBuffer {
    /// Allocate `len` bytes of virtually contiguous kernel memory.
    pub fn new(len: usize) -> Result<Self> {
        pr_info!("vmalloc: requesting {} bytes\n", len);
        // Real code:
        //   let ptr = unsafe { bindings::vmalloc(len) };
        //   if ptr.is_null() { return Err(ENOMEM); }
        Ok(Self { len })
    }
}

impl Drop for VmallocBuffer {
    fn drop(&mut self) {
        pr_info!("vmalloc: freeing {} bytes\n", self.len);
        // Real code: unsafe { bindings::vfree(self.ptr); }
    }
}

// ---------------------------------------------------------------------------
// Memory allocator pattern summary (comments only)
// ---------------------------------------------------------------------------
//
// Rule of thumb for choosing an allocator:
//
//  Object size  | Physically contiguous? | DMA?  | Allocator
//  -------------|------------------------|-------|------------------------
//  <= ~4 MiB    | Required               | Maybe | kmalloc (GFP_KERNEL/DMA)
//  <= ~4 MiB    | Not required           | No    | kmalloc (GFP_KERNEL)
//  >  ~4 MiB    | Not required           | No    | vmalloc
//  Many same-sz | Irrelevant             | No    | kmem_cache (slab)
//  Page aligned | Required               | Yes   | alloc_pages + dma_map
//
// In Rust, always prefer the safe wrappers in the `kernel` crate when they
// exist. Fall back to `unsafe` + `bindings::*` only when no safe wrapper is
// available, and document the invariants carefully.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_buffer_size() {
        // KernelBuffer holds one NonNull<u8> (8 bytes) + one usize (8 bytes).
        // This test ensures the struct layout is as expected.
        // In-kernel tests would use the KUnit framework instead.
        assert_eq!(core::mem::size_of::<KernelBuffer>(), 16);
    }

    #[test]
    fn test_kernel_buffer_write_read() {
        let mut buf = KernelBuffer::new(64).expect("allocation failed");
        buf.write_byte(0, 0xAB);
        buf.write_byte(63, 0xCD);
        assert_eq!(buf.read_byte(0), 0xAB);
        assert_eq!(buf.read_byte(63), 0xCD);
    }

    #[test]
    #[should_panic]
    fn test_kernel_buffer_out_of_bounds_read() {
        let buf = KernelBuffer::new(8).expect("allocation failed");
        let _ = buf.read_byte(8); // must panic
    }

    #[test]
    #[should_panic]
    fn test_kernel_buffer_out_of_bounds_write() {
        let mut buf = KernelBuffer::new(8).expect("allocation failed");
        buf.write_byte(8, 0xFF); // must panic
    }
}
