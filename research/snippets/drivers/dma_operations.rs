// SPDX-License-Identifier: GPL-2.0

//! DMA and Hardware Interaction Safety Patterns
//!
//! Direct Memory Access (DMA) lets hardware peripherals read/write system
//! memory without CPU involvement. Getting DMA right is notoriously
//! difficult in C; Rust's type system can enforce many of the invariants
//! statically.
//!
//! ## Concepts Covered
//!
//! - Coherent (consistent) vs. streaming DMA
//! - IOMMU and address translation
//! - Cache coherency and memory barriers
//! - Safe wrappers that prevent common DMA bugs
//! - Hardware register access via MMIO
//! - Device ownership and lifetime rules
//!
//! ## Common DMA Bugs (that Rust helps prevent)
//!
//! 1. **Use-after-free**: CPU accesses buffer while device is still writing
//! 2. **Double-map**: Same buffer mapped twice with incompatible directions
//! 3. **Direction mismatch**: Buffer mapped for `TO_DEVICE` but CPU writes after sync
//! 4. **Missing sync**: CPU reads stale cached data after device DMA write
//! 5. **Alias**: CPU virtual alias and DMA physical address map different cache lines
//!
//! ## Build
//!
//! Place in `samples/rust/` and enable `CONFIG_RUST=y`.
//!
//! ## Difficulty
//!
//! Advanced

use kernel::prelude::*;

// ---------------------------------------------------------------------------
// 1. DMA direction type — enforced at the type level
// ---------------------------------------------------------------------------

/// Represents the direction of a DMA transfer.
///
/// Encoding this in the type prevents using a buffer mapped for one direction
/// in an incompatible way (e.g., the device writing into a `ToDevice` buffer
/// and the CPU reading stale data without a sync).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// CPU writes, device reads — `DMA_TO_DEVICE`
    ToDevice,
    /// Device writes, CPU reads — `DMA_FROM_DEVICE`
    FromDevice,
    /// Both sides can read and write — `DMA_BIDIRECTIONAL`
    Bidirectional,
}

impl DmaDirection {
    /// Map to the kernel's `dma_data_direction` enum value.
    pub fn to_kernel_constant(self) -> u32 {
        match self {
            DmaDirection::ToDevice => 1,      // DMA_TO_DEVICE
            DmaDirection::FromDevice => 2,    // DMA_FROM_DEVICE
            DmaDirection::Bidirectional => 0, // DMA_BIDIRECTIONAL
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Coherent (consistent) DMA allocation
// ---------------------------------------------------------------------------

/// A coherent DMA buffer allocated with `dma_alloc_coherent`.
///
/// Coherent memory is simultaneously accessible by both the CPU and the
/// device without explicit cache flushes. This is the easiest DMA type to
/// use correctly, but it is also the most expensive on platforms that lack
/// hardware coherency (e.g., some ARM SoCs).
///
/// The kernel allocates a physically contiguous buffer and sets up both:
/// - A CPU virtual address (for kernel access)
/// - A DMA address (for the device's address space, translated by IOMMU)
///
/// **Ownership rule**: The buffer must NOT be freed while the device can
/// still access it. The `Drop` impl here enforces this by requiring the
/// device to be explicitly detached first.
pub struct CoherentDmaBuffer {
    /// CPU-accessible virtual address.
    cpu_addr: core::ptr::NonNull<u8>,
    /// Device-visible DMA address (after IOMMU translation).
    dma_addr: u64,
    /// Size in bytes.
    size: usize,
}

// SAFETY: DMA buffers can be shared across cores; access is serialised by
// the driver's locking strategy.
unsafe impl Send for CoherentDmaBuffer {}
unsafe impl Sync for CoherentDmaBuffer {}

impl CoherentDmaBuffer {
    /// Allocate a coherent DMA buffer of `size` bytes.
    ///
    /// In a real driver this calls:
    /// ```c
    /// cpu_addr = dma_alloc_coherent(dev, size, &dma_addr, GFP_KERNEL);
    /// ```
    pub fn alloc(size: usize) -> Result<Self> {
        // Placeholder: use Box to represent the allocation.
        // Real code would call bindings::dma_alloc_coherent.
        let boxed = Box::try_new_slice(size, 0u8)?;
        let cpu_ptr = Box::into_raw(boxed) as *mut u8;

        // SAFETY: Box::into_raw always returns non-null.
        let cpu_addr = unsafe { core::ptr::NonNull::new_unchecked(cpu_ptr) };
        let dma_addr = cpu_ptr as u64; // Real code: IOMMU-translated address

        pr_info!("Coherent DMA buffer: cpu={:p}, dma=0x{:x}, size={}\n",
            cpu_addr.as_ptr(), dma_addr, size);

        Ok(Self { cpu_addr, dma_addr, size })
    }

    /// Return the DMA address to program into the device's descriptor ring.
    pub fn dma_addr(&self) -> u64 {
        self.dma_addr
    }

    /// Return a CPU-side slice for reading/writing the buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the device is not currently writing to the
    /// buffer. For coherent memory, no explicit flush is needed.
    pub unsafe fn as_slice(&self) -> &[u8] {
        // SAFETY: cpu_addr is valid for `size` bytes.
        core::slice::from_raw_parts(self.cpu_addr.as_ptr(), self.size)
    }

    /// Return a mutable CPU-side slice for writing.
    ///
    /// # Safety
    ///
    /// The caller must ensure the device is not reading or writing the buffer
    /// concurrently.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: cpu_addr is valid for `size` bytes; we hold &mut self.
        core::slice::from_raw_parts_mut(self.cpu_addr.as_ptr(), self.size)
    }
}

impl Drop for CoherentDmaBuffer {
    fn drop(&mut self) {
        pr_info!("Freeing coherent DMA buffer at dma=0x{:x}\n", self.dma_addr);
        // Real code: bindings::dma_free_coherent(dev, size, cpu_addr, dma_addr);
        //
        // Reconstruct Box to free the placeholder allocation.
        // SAFETY: cpu_addr came from Box::into_raw.
        unsafe {
            let slice_ptr =
                core::ptr::slice_from_raw_parts_mut(self.cpu_addr.as_ptr(), self.size);
            drop(Box::from_raw(slice_ptr));
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Streaming DMA (mapped from existing buffer)
// ---------------------------------------------------------------------------

/// A streaming DMA mapping wrapping an existing CPU buffer.
///
/// Streaming mappings are created on-the-fly for individual transfers and
/// must be explicitly synchronised between CPU and device accesses:
///
/// - Before the device reads: call `sync_for_device()` (flush CPU caches)
/// - After the device writes: call `sync_for_cpu()` (invalidate CPU caches)
///
/// The type parameter `D` encodes the direction at compile time, preventing
/// incorrect sync direction mismatches.
pub struct StreamingDmaMapping<const DIR: u32> {
    dma_addr: u64,
    size: usize,
    /// Tracks whether the mapping is currently "owned" by the device.
    device_owned: bool,
}

impl<const DIR: u32> StreamingDmaMapping<DIR> {
    /// Map `size` bytes starting at `cpu_phys_addr` for DMA.
    ///
    /// Real code: `dma_map_single(dev, virt_addr, size, direction)`
    pub fn map(cpu_phys_addr: usize, size: usize) -> Result<Self> {
        // Stub: in real code, IOMMU translates cpu_phys_addr → dma_addr.
        let dma_addr = cpu_phys_addr as u64;
        pr_info!("Streaming DMA map: addr=0x{:x}, size={}, dir={}\n",
            dma_addr, size, DIR);
        Ok(Self {
            dma_addr,
            size,
            device_owned: false,
        })
    }

    /// Transfer ownership to the device: flush CPU caches.
    ///
    /// After this call, the CPU must not touch the buffer until
    /// `sync_for_cpu()` is called.
    pub fn sync_for_device(&mut self) {
        // Real code: dma_sync_single_for_device(dev, dma_addr, size, dir)
        pr_info!("DMA sync for device: addr=0x{:x}\n", self.dma_addr);
        self.device_owned = true;
    }

    /// Transfer ownership back to the CPU: invalidate stale caches.
    ///
    /// After this call, the CPU can safely read data written by the device.
    pub fn sync_for_cpu(&mut self) {
        // Real code: dma_sync_single_for_cpu(dev, dma_addr, size, dir)
        pr_info!("DMA sync for CPU: addr=0x{:x}\n", self.dma_addr);
        self.device_owned = false;
    }

    /// Return the DMA address to program into the hardware descriptor.
    pub fn dma_addr(&self) -> u64 {
        self.dma_addr
    }
}

impl<const DIR: u32> Drop for StreamingDmaMapping<DIR> {
    fn drop(&mut self) {
        // Warn if dropped while device still owns the buffer.
        if self.device_owned {
            pr_warn!("StreamingDmaMapping dropped while device-owned!\n");
        }
        // Real code: dma_unmap_single(dev, dma_addr, size, direction);
        pr_info!("Streaming DMA unmap: addr=0x{:x}\n", self.dma_addr);
    }
}

// ---------------------------------------------------------------------------
// 4. Memory-Mapped I/O (MMIO) register access
// ---------------------------------------------------------------------------

/// A safe, typed wrapper over a hardware register block.
///
/// MMIO registers must be accessed with volatile operations to prevent the
/// compiler from optimising away or reordering reads/writes. The kernel
/// provides `readl`/`writel` (and variants) which combine:
/// - Volatile memory access
/// - An appropriate memory barrier (DSB / MFENCE on x86)
///
/// Rust's `core::ptr::read_volatile` / `write_volatile` correspond directly
/// to these, but the kernel's `ioread32`/`iowrite32` are preferred in real
/// drivers because they also handle endianness and I/O vs memory space.
pub struct MmioBlock {
    base: core::ptr::NonNull<u8>,
    size: usize,
}

// SAFETY: MMIO regions are global hardware state; sharing is safe given
// the driver serialises concurrent accesses.
unsafe impl Send for MmioBlock {}

impl MmioBlock {
    /// Map a hardware register block at `phys_addr` of `size` bytes.
    ///
    /// Real code: `ioremap(phys_addr, size)` followed by a null check.
    ///
    /// # Safety
    ///
    /// `phys_addr` must be a valid MMIO physical address for this system.
    /// The size must not exceed the device's register space.
    pub unsafe fn new(phys_addr: usize, size: usize) -> Result<Self> {
        // Stub: cast address to pointer (real code calls ioremap).
        if phys_addr == 0 {
            return Err(EINVAL);
        }
        let ptr = phys_addr as *mut u8;
        let base = core::ptr::NonNull::new(ptr).ok_or(ENOMEM)?;
        pr_info!("MMIO mapped: phys=0x{:x}, size={}\n", phys_addr, size);
        Ok(Self { base, size })
    }

    /// Read a 32-bit register at `offset` bytes from the base.
    ///
    /// # Safety
    ///
    /// `offset` must be 4-byte aligned and within `[0, size - 4]`.
    pub unsafe fn read32(&self, offset: usize) -> u32 {
        debug_assert!(offset + 4 <= self.size, "offset out of MMIO range");
        debug_assert!(offset % 4 == 0, "unaligned MMIO read");
        // SAFETY: offset is in range; caller guarantees validity.
        let ptr = self.base.as_ptr().add(offset) as *const u32;
        core::ptr::read_volatile(ptr)
    }

    /// Write a 32-bit value to a register at `offset` bytes from the base.
    ///
    /// # Safety
    ///
    /// `offset` must be 4-byte aligned and within `[0, size - 4]`.
    pub unsafe fn write32(&self, offset: usize, value: u32) {
        debug_assert!(offset + 4 <= self.size, "offset out of MMIO range");
        debug_assert!(offset % 4 == 0, "unaligned MMIO write");
        // SAFETY: offset is in range; caller guarantees validity.
        let ptr = self.base.as_ptr().add(offset) as *mut u32;
        core::ptr::write_volatile(ptr, value);
    }
}

impl Drop for MmioBlock {
    fn drop(&mut self) {
        pr_info!("MMIO unmapped\n");
        // Real code: iounmap(self.base.as_ptr());
    }
}

// ---------------------------------------------------------------------------
// 5. Descriptor ring pattern (used in network/NVMe/GPU drivers)
// ---------------------------------------------------------------------------

/// A single entry in a DMA descriptor ring.
///
/// Hardware descriptor rings are the primary mechanism for high-throughput
/// DMA in network, storage, and GPU drivers. Each entry contains the DMA
/// address, length, and control flags for one transfer.
///
/// `#[repr(C)]` is mandatory because the hardware interprets the layout
/// directly.
#[repr(C)]
pub struct DmaDescriptor {
    /// Physical (DMA) address of the buffer.
    pub addr: u64,
    /// Length of the transfer in bytes.
    pub len: u32,
    /// Hardware control flags (device-specific).
    pub flags: u32,
}

impl DmaDescriptor {
    /// Create a descriptor for a coherent buffer.
    pub fn new(buffer: &CoherentDmaBuffer, len: u32, flags: u32) -> Self {
        Self {
            addr: buffer.dma_addr(),
            len,
            flags,
        }
    }
}

// SAFETY: repr(C) layout is preserved; verify size at compile time.
const _: () = assert!(core::mem::size_of::<DmaDescriptor>() == 16);

// ---------------------------------------------------------------------------
// DMA safety checklist (comments)
// ---------------------------------------------------------------------------
//
//  ✅  Always check dma_mapping_error() after dma_map_*() calls.
//  ✅  Respect the direction: only sync in the direction specified at map time.
//  ✅  Free coherent buffers with dma_free_coherent(), not kfree().
//  ✅  Use dma_set_mask_and_coherent() to configure the device's DMA mask.
//  ✅  Never touch streaming buffers between sync_for_device() and sync_for_cpu().
//  ✅  Use IOMMU when available — it provides address translation and protection.
//  ✅  For PCIe, prefer 64-bit DMA addresses when the device supports them.
//  ✅  Use write_volatile / read_volatile (or iowrite*/ioread*) for MMIO.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_direction_constants() {
        assert_eq!(DmaDirection::Bidirectional.to_kernel_constant(), 0);
        assert_eq!(DmaDirection::ToDevice.to_kernel_constant(), 1);
        assert_eq!(DmaDirection::FromDevice.to_kernel_constant(), 2);
    }

    #[test]
    fn test_descriptor_size() {
        assert_eq!(core::mem::size_of::<DmaDescriptor>(), 16);
    }

    #[test]
    fn test_coherent_alloc_and_drop() {
        // This test exercises the placeholder allocation path (not real DMA).
        let buf = CoherentDmaBuffer::alloc(256);
        assert!(buf.is_ok());
        // buf is freed when it goes out of scope.
    }
}
