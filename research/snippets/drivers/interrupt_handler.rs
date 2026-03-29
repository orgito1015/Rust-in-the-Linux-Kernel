// SPDX-License-Identifier: GPL-2.0

//! Interrupt Handler Patterns in Rust for the Linux Kernel
//!
//! Interrupt handlers (IRQ handlers) are called by the kernel when hardware
//! signals an event. They run in a special context with severe restrictions:
//!
//! - **No sleeping**: cannot call `schedule()`, `mutex_lock()`, or any
//!   blocking allocator (`GFP_KERNEL`). Use `GFP_ATOMIC` when allocation
//!   is unavoidable.
//! - **No user-space access**: cannot call `copy_to_user()`.
//! - **Minimal work**: defer heavy processing to a bottom half
//!   (tasklet, softirq, workqueue, or threaded IRQ).
//! - **Shared interrupts**: handlers must return `IRQ_NONE` if the interrupt
//!   was not from their device.
//!
//! ## Patterns Demonstrated
//!
//! 1. Simple IRQ handler returning `IRQ_HANDLED`/`IRQ_NONE`
//! 2. Top-half / bottom-half split with a workqueue
//! 3. Threaded interrupt handler (`request_threaded_irq`)
//! 4. Per-device private data access in a handler
//! 5. Shared interrupt lines
//!
//! ## Build
//!
//! Place in `samples/rust/` and enable `CONFIG_RUST=y`.
//!
//! ## Difficulty
//!
//! Advanced

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use kernel::prelude::*;

// ---------------------------------------------------------------------------
// IRQ return values
// ---------------------------------------------------------------------------

/// Mirrors the kernel's `irqreturn_t`.
///
/// An IRQ handler must return one of these values so the kernel knows whether
/// the interrupt was handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IrqReturn {
    /// This interrupt was not from our device — another handler should try.
    None = 0,
    /// We handled the interrupt.
    Handled = 1,
    /// We handled the interrupt and want a threaded handler to run next.
    WakeThread = 2,
}

// ---------------------------------------------------------------------------
// 1. Simple top-half handler
// ---------------------------------------------------------------------------

/// Per-device state shared between the ISR and the rest of the driver.
///
/// Only atomics and spinlock-protected data are safe to access from IRQ
/// context. Using `AtomicU32` / `AtomicBool` here avoids any locking.
pub struct SimpleDeviceData {
    /// Incremented each time an interrupt fires.
    pub irq_count: AtomicU32,
    /// Set to `true` when the device signals "data ready".
    pub data_ready: AtomicBool,
}

impl SimpleDeviceData {
    /// Create zeroed device data.
    pub const fn new() -> Self {
        Self {
            irq_count: AtomicU32::new(0),
            data_ready: AtomicBool::new(false),
        }
    }
}

impl Default for SimpleDeviceData {
    fn default() -> Self {
        Self::new()
    }
}

/// A minimal IRQ handler.
///
/// In a real driver this function would be registered with
/// `request_irq(irq, handler, flags, name, dev_id)`.
///
/// # Safety
///
/// - Must be called only in IRQ context.
/// - `data` must point to a valid, properly aligned `SimpleDeviceData`.
/// - No sleeping, no blocking allocations.
///
/// # Return
///
/// - `IrqReturn::Handled` if the interrupt belonged to our device.
/// - `IrqReturn::None` if it did not (shared IRQ line).
pub unsafe extern "C" fn simple_irq_handler(
    _irq: u32,
    data: *mut core::ffi::c_void,
) -> IrqReturn {
    // SAFETY: data is the dev_id pointer we passed to request_irq.
    let dev = &*(data as *const SimpleDeviceData);

    // Read a hypothetical hardware status register.
    // In real code: let status = ioread32(dev.base + STATUS_REG);
    let status: u32 = 0x0001; // simulated "interrupt pending" bit

    if status & 0x0001 == 0 {
        // Interrupt was not from our device.
        return IrqReturn::None;
    }

    // Acknowledge the interrupt to the hardware (clear the pending bit).
    // Real code: iowrite32(dev.base + ACK_REG, status);

    dev.irq_count.fetch_add(1, Ordering::Relaxed);
    dev.data_ready.store(true, Ordering::Release);

    // Wake up any process waiting for data (would be done via a wait queue
    // in real code: wake_up_interruptible(&dev->wq);).

    IrqReturn::Handled
}

// ---------------------------------------------------------------------------
// 2. Top-half / bottom-half split with a workqueue
// ---------------------------------------------------------------------------

/// Represents a deferred work item posted by the top-half ISR.
///
/// Workqueues run in process context, so they *can* sleep. This makes them
/// suitable for:
/// - DMA buffer processing
/// - Filesystem operations
/// - Network stack push
/// - Any I/O that might block
///
/// The kernel API:
/// ```c
/// INIT_WORK(&dev->work, bottom_half_fn);
/// schedule_work(&dev->work);   // called from IRQ handler
/// ```
pub struct WorkqueueExample {
    irq_count: AtomicU32,
    work_queued: AtomicBool,
}

impl WorkqueueExample {
    pub const fn new() -> Self {
        Self {
            irq_count: AtomicU32::new(0),
            work_queued: AtomicBool::new(false),
        }
    }

    /// **Top half** — runs in hard IRQ context.
    ///
    /// Records the event and schedules the bottom half.
    /// Must return quickly; no heavy processing here.
    pub fn top_half(&self) -> IrqReturn {
        self.irq_count.fetch_add(1, Ordering::Relaxed);

        // Schedule the workqueue item (non-blocking).
        // Real code: schedule_work(&self.work);
        self.work_queued.store(true, Ordering::Release);

        IrqReturn::Handled
    }

    /// **Bottom half** — runs in process context (workqueue thread).
    ///
    /// Can sleep, allocate with `GFP_KERNEL`, and call blocking APIs.
    pub fn bottom_half(&self) {
        if !self.work_queued.swap(false, Ordering::AcqRel) {
            return;
        }

        pr_info!("Bottom half: processing {} interrupts\n",
            self.irq_count.load(Ordering::Relaxed));

        // Perform expensive processing here:
        // - Read DMA buffer
        // - Parse packet header
        // - Submit to network/block layer
        // - Update statistics
    }
}

impl Default for WorkqueueExample {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 3. Threaded interrupt handler
// ---------------------------------------------------------------------------

/// Demonstrates the threaded IRQ pattern.
///
/// `request_threaded_irq()` registers two functions:
/// 1. **Primary handler** — runs in hard IRQ context; must be fast.
///    Returns `IRQ_WAKE_THREAD` to defer to the threaded handler.
/// 2. **Thread handler** — runs in a dedicated kernel thread; can sleep.
///
/// This is the preferred modern pattern for drivers that need process context
/// but want the simplicity of a single IRQ handler.
pub struct ThreadedIrqDevice {
    pending: AtomicBool,
    packets_processed: AtomicU32,
}

impl ThreadedIrqDevice {
    pub const fn new() -> Self {
        Self {
            pending: AtomicBool::new(false),
            packets_processed: AtomicU32::new(0),
        }
    }

    /// **Primary handler** — called in hard-IRQ context.
    ///
    /// Should only:
    /// 1. Check whether the interrupt is ours.
    /// 2. Mask the interrupt at the hardware level (prevent re-triggering).
    /// 3. Return `IrqReturn::WakeThread`.
    pub fn primary_handler(&self) -> IrqReturn {
        // Check device status register (simulated).
        let our_interrupt = true; // In real code: read status register

        if !our_interrupt {
            return IrqReturn::None;
        }

        // Disable the interrupt source to prevent flooding while we process.
        // Real code: iowrite32(dev.base + IRQ_MASK_REG, 0);

        self.pending.store(true, Ordering::Release);
        IrqReturn::WakeThread
    }

    /// **Thread handler** — called in process context (kernel thread).
    ///
    /// Safe to sleep, allocate, and perform heavy work.
    pub fn thread_handler(&self) -> IrqReturn {
        if !self.pending.swap(false, Ordering::AcqRel) {
            return IrqReturn::None;
        }

        // Process received data (can sleep here).
        pr_info!("Threaded handler: processing IRQ\n");
        self.packets_processed.fetch_add(1, Ordering::Relaxed);

        // Re-enable the interrupt source.
        // Real code: iowrite32(dev.base + IRQ_MASK_REG, IRQ_ENABLE);

        IrqReturn::Handled
    }
}

impl Default for ThreadedIrqDevice {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 4. Accessing per-device data safely from a handler
// ---------------------------------------------------------------------------

/// Demonstrates the correct pattern for per-device private data.
///
/// The kernel passes the `dev_id` pointer (registered with `request_irq`) to
/// the handler. Rust enforces that:
/// - The pointer is cast to the correct type.
/// - All shared state uses appropriate synchronisation (`AtomicXxx` or
///   spinlocks for IRQ-safe access).
///
/// **Never** use `Mutex` in an IRQ handler — it can sleep.
/// **Always** use `SpinLock` (with IRQ save) or atomics.
pub struct PerDeviceIrqData {
    rx_packets: AtomicU32,
    tx_packets: AtomicU32,
    error_count: AtomicU32,
}

impl PerDeviceIrqData {
    pub const fn new() -> Self {
        Self {
            rx_packets: AtomicU32::new(0),
            tx_packets: AtomicU32::new(0),
            error_count: AtomicU32::new(0),
        }
    }

    /// IRQ-safe statistics accessor.
    pub fn on_rx(&self) {
        self.rx_packets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn on_tx(&self) {
        self.tx_packets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn on_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stats(&self) -> (u32, u32, u32) {
        (
            self.rx_packets.load(Ordering::Relaxed),
            self.tx_packets.load(Ordering::Relaxed),
            self.error_count.load(Ordering::Relaxed),
        )
    }
}

impl Default for PerDeviceIrqData {
    fn default() -> Self {
        Self::new()
    }
}

/// Example of casting the `dev_id` void pointer back to our data.
///
/// # Safety
///
/// `data` must be the same pointer that was passed to `request_irq`, and it
/// must remain valid until `free_irq` is called.
pub unsafe extern "C" fn per_device_irq_handler(
    _irq: u32,
    data: *mut core::ffi::c_void,
) -> IrqReturn {
    // SAFETY: data is our PerDeviceIrqData, valid for the lifetime of the driver.
    let dev = &*(data as *const PerDeviceIrqData);

    // Simulate reading a hardware interrupt cause register.
    let cause: u32 = 0b0011; // bits 0=RX, 1=TX, 2=ERR

    if cause == 0 {
        return IrqReturn::None;
    }

    if cause & 0b0001 != 0 {
        dev.on_rx();
    }
    if cause & 0b0010 != 0 {
        dev.on_tx();
    }
    if cause & 0b0100 != 0 {
        dev.on_error();
    }

    IrqReturn::Handled
}

// ---------------------------------------------------------------------------
// IRQ handler rules summary (comments)
// ---------------------------------------------------------------------------
//
//  ✅  Return IRQ_NONE when the interrupt is not from your device.
//  ✅  Acknowledge (ACK) the interrupt before returning IRQ_HANDLED.
//  ✅  Use only atomic ops or spinlocks (with IRQs disabled) for shared state.
//  ✅  Defer heavy work to workqueues or threaded handlers.
//  ✅  Never call schedule(), msleep(), wait_event(), or mutex_lock().
//  ✅  Never allocate with GFP_KERNEL; use GFP_ATOMIC if you must allocate.
//  ✅  Keep the handler as short as possible to minimise interrupt latency.
//  ✅  Use IRQF_SHARED flag when sharing an IRQ line with other devices.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irq_return_values() {
        assert_eq!(IrqReturn::None as u32, 0);
        assert_eq!(IrqReturn::Handled as u32, 1);
        assert_eq!(IrqReturn::WakeThread as u32, 2);
    }

    #[test]
    fn test_simple_device_data() {
        let data = SimpleDeviceData::new();
        assert_eq!(data.irq_count.load(Ordering::Relaxed), 0);
        assert!(!data.data_ready.load(Ordering::Relaxed));
    }

    #[test]
    fn test_top_half_bottom_half() {
        let dev = WorkqueueExample::new();
        let ret = dev.top_half();
        assert_eq!(ret, IrqReturn::Handled);
        assert_eq!(dev.irq_count.load(Ordering::Relaxed), 1);
        assert!(dev.work_queued.load(Ordering::Relaxed));
        dev.bottom_half();
        assert!(!dev.work_queued.load(Ordering::Relaxed));
    }

    #[test]
    fn test_threaded_irq() {
        let dev = ThreadedIrqDevice::new();
        let ret = dev.primary_handler();
        assert_eq!(ret, IrqReturn::WakeThread);
        let ret = dev.thread_handler();
        assert_eq!(ret, IrqReturn::Handled);
        assert_eq!(dev.packets_processed.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_per_device_stats() {
        let data = PerDeviceIrqData::new();
        data.on_rx();
        data.on_rx();
        data.on_tx();
        data.on_error();
        let (rx, tx, err) = data.stats();
        assert_eq!(rx, 2);
        assert_eq!(tx, 1);
        assert_eq!(err, 1);
    }
}
