# Technical Deep Dives

This document provides in-depth coverage of the five most important low-level patterns for writing
correct and safe Rust code in the Linux kernel:

1. [Memory Allocator Integration Patterns](#1-memory-allocator-integration-patterns)
2. [Lock-Free Data Structure Implementations](#2-lock-free-data-structure-implementations)
3. [DMA and Hardware Interaction Safety](#3-dma-and-hardware-interaction-safety)
4. [Interrupt Handler Patterns in Rust](#4-interrupt-handler-patterns-in-rust)
5. [Error Propagation Mechanisms](#5-error-propagation-mechanisms)

Each section describes the concept, shows the key Rust idioms, and links to the corresponding
runnable code snippet.

---

## 1. Memory Allocator Integration Patterns

**Code snippet**: [`research/snippets/memory/memory_allocator.rs`](../research/snippets/memory/memory_allocator.rs)

### Allocator Overview

### The Kernel Allocators

| Allocator | API | Use case |
|-----------|-----|----------|
| SLUB/SLAB | `kmalloc` / `kfree` | General objects ≤ 4 MiB |
| Vmalloc | `vmalloc` / `vfree` | Large, non-contiguous buffers |
| Page allocator | `alloc_pages` / `free_pages` | Page-aligned, physically contiguous |
| Slab cache | `kmem_cache_create` / `kmem_cache_alloc` | Many same-sized objects |
| Per-CPU | `alloc_percpu` / `free_percpu` | Per-CPU variables |

### GFP Flags

GFP (Get Free Pages) flags control *how* an allocation is performed:

```rust
// Process context — may sleep and reclaim memory
let buf = Box::try_new([0u8; 128])?;  // GFP_KERNEL under the hood

// Interrupt context — must NOT sleep
// Real code: kmalloc(size, GFP_ATOMIC)
// The kernel crate exposes this via a separate allocator type.
```

Key flags:

- **`GFP_KERNEL`**: Standard; may sleep. Use in process context.
- **`GFP_ATOMIC`**: Non-blocking; may fail. Use in IRQ/softirq context.
- **`GFP_DMA`** / **`GFP_DMA32`**: Restrict to DMA-addressable memory zones.
- **`GFP_NOWAIT`**: Like `GFP_ATOMIC` but without emergency reserves.

### Rust Mapping

```rust
// C
void *p = kmalloc(sizeof(struct foo), GFP_KERNEL);
if (!p) return -ENOMEM;
// ... use p ...
kfree(p);

// Rust equivalent — no manual free needed
let p: Box<Foo> = Box::try_new(Foo::default())?;
// p is automatically freed when it goes out of scope
```

### Slab Caches

When many objects of the same type are allocated and freed frequently, a dedicated slab cache
(`kmem_cache`) is faster and reduces fragmentation:

```rust
// Create the cache once at module init
let cache = SlabCache::new(c_str!("my_objects"), size_of::<MyObject>())?;

// Allocate from it in the hot path
let obj_ptr = cache.alloc()?;

// Free individual objects back to the cache
// kmem_cache_free(cache, obj_ptr);

// Cache is destroyed automatically when SlabCache is dropped
```

### Guidelines

- Always use `try_new` / `try_with_capacity` variants — they return `Result` rather than panicking.
- Prefer `Box<T>` / `Vec<T>` from the kernel crate over raw pointer manipulation.
- Use `vmalloc` only when you need large, virtually-contiguous memory and physical
  contiguity is not required.
- Use slab caches for hot-path allocations of fixed-size objects.

---

## 2. Lock-Free Data Structure Implementations

**Code snippet**: [`research/snippets/concurrency/lock_free.rs`](../research/snippets/concurrency/lock_free.rs)

### Lock-Free Overview

Lock-free algorithms improve scalability on multi-core systems by eliminating mutex contention.
Rust's `core::sync::atomic` module provides the building blocks: `AtomicBool`, `AtomicU32`,
`AtomicUsize`, `AtomicPtr`, and the compare-and-swap (CAS) primitive.

### Memory Ordering

Choosing the correct `Ordering` is critical for correctness:

```rust
use core::sync::atomic::Ordering;

// Relaxed — atomicity only; no ordering guarantees.
// Good for independent counters.
counter.fetch_add(1, Ordering::Relaxed);

// Acquire — all subsequent loads/stores happen AFTER this load.
// Used on the reader side of a producer/consumer protocol.
let value = ptr.load(Ordering::Acquire);

// Release — all preceding loads/stores happen BEFORE this store.
// Used on the writer side.
ptr.store(new_value, Ordering::Release);

// AcqRel — combined Acquire + Release for read-modify-write operations.
let old = ptr.compare_exchange(expected, new, Ordering::AcqRel, Ordering::Acquire);

// SeqCst — total sequential consistency; most expensive.
// Use only when multiple independent atomics must be observed in order.
```

### Treiber Lock-Free Stack

The Treiber stack is the canonical lock-free stack. It uses a single `AtomicPtr` for the head
and a CAS loop to push and pop without a mutex:

```rust
// Push: allocate node, CAS head to point to it
pub fn push(&self, value: T) -> Result<()> {
    let node_ptr = Box::into_raw(Box::try_new(Node { value, next: null_mut() })?);
    loop {
        let old_head = self.head.load(Ordering::Relaxed);
        unsafe { (*node_ptr).next = old_head };
        if self.head.compare_exchange_weak(old_head, node_ptr,
            Ordering::Release, Ordering::Relaxed).is_ok() {
            return Ok(());
        }
    }
}

// Pop: CAS head to point to head->next
pub fn pop(&self) -> Option<T> {
    loop {
        let head = self.head.load(Ordering::Acquire);
        if head.is_null() { return None; }
        let next = unsafe { (*head).next };
        if self.head.compare_exchange_weak(head, next,
            Ordering::AcqRel, Ordering::Acquire).is_ok() {
            let node = unsafe { Box::from_raw(head) };
            return Some(node.value);
        }
    }
}
```

### Atomic Reference Counting

Rust provides `Arc<T>` in `std`, but the kernel uses a custom implementation backed by
`refcount_t` (which has overflow protection). The key insight is correct memory ordering:

```rust
// Clone: increment with Relaxed (the Acquire in drop synchronises)
refcount.fetch_add(1, Ordering::Relaxed);

// Drop: decrement with Release; fence on reaching 0
if refcount.fetch_sub(1, Ordering::Release) == 1 {
    fence(Ordering::Acquire);  // See all writes from previous owners
    // Now safe to free
}
```

### RCU (Read-Copy-Update)

RCU is the kernel's highest-throughput read-mostly data structure mechanism:

- **Readers**: use `rcu_read_lock()` / `rcu_read_unlock()` — effectively free on non-preemptible kernels.
- **Writers**: copy the data, update the copy, atomically publish the new pointer with
  `rcu_assign_pointer()`, then call `synchronize_rcu()` to wait for all existing readers to finish
  before freeing the old copy.

```rust
// Reader (maps to rcu_dereference + rcu_read_lock/unlock)
protected.read(|value| {
    process(value);  // Safe read-only access
});

// Writer (maps to rcu_assign_pointer + synchronize_rcu + kfree_rcu)
let old_ptr = unsafe { protected.replace(new_value)? };
// synchronize_rcu() here (waits for all readers)
// unsafe { drop(Box::from_raw(old_ptr)); }
```

### When to Use Each Approach

| Approach | Throughput | Latency | Use case |
|----------|-----------|---------|----------|
| Mutex | Low | Variable | Infrequent writes, complex invariants |
| Spinlock | Medium | Low | Short critical sections, IRQ context |
| Atomic ops | High | Very low | Single-value flags, counters |
| Lock-free stack | High | Low | Producer/consumer, work queues |
| RCU | Very high (reads) | Low (reads) | Read-mostly global data |

---

## 3. DMA and Hardware Interaction Safety

**Code snippet**: [`research/snippets/drivers/dma_operations.rs`](../research/snippets/drivers/dma_operations.rs)

### DMA Overview

Direct Memory Access (DMA) is how hardware peripherals read and write system memory without CPU
involvement. Getting DMA right is notoriously difficult, but Rust's type system can enforce key
safety invariants at compile time.

### DMA Types

**Coherent (consistent) DMA**

Memory that is simultaneously coherent for both the CPU and the device. No explicit cache
flushes are needed. More expensive to allocate; use for control structures and descriptor rings.

```rust
// C
void *cpu_addr = dma_alloc_coherent(dev, size, &dma_addr, GFP_KERNEL);
// ...
dma_free_coherent(dev, size, cpu_addr, dma_addr);

// Rust (RAII wrapper)
let buf = CoherentDmaBuffer::alloc(size)?;
let dma_addr = buf.dma_addr(); // Program into device registers
// buf is freed automatically via Drop
```

**Streaming DMA**

A CPU buffer is temporarily mapped for a single DMA transfer and then unmapped. Cache sync calls
are required to maintain coherency between CPU and device views of the data:

```rust
// Before device reads: flush CPU caches
mapping.sync_for_device();
// Start DMA transfer on hardware
// ...
// After device writes: invalidate stale CPU caches
mapping.sync_for_cpu();
let data = unsafe { slice::from_raw_parts(cpu_ptr, size) };
```

### DMA Descriptor Rings

Descriptor rings are the primary mechanism for high-throughput DMA (network, NVMe, GPU drivers):

```rust
#[repr(C)]  // Hardware reads this layout directly
struct DmaDescriptor {
    addr:  u64,  // DMA address of the buffer
    len:   u32,  // Transfer length
    flags: u32,  // Device-specific control bits
}

// Verify at compile time that the size matches hardware specification
const _: () = assert!(core::mem::size_of::<DmaDescriptor>() == 16);
```

### Memory-Mapped I/O (MMIO)

Hardware registers are accessed through memory-mapped addresses. **Always** use volatile
operations — the compiler must not cache or reorder these accesses:

```rust
// C
writel(value, base + REGISTER_OFFSET);
u32 val = readl(base + REGISTER_OFFSET);

// Rust
unsafe {
    let ptr = base.add(REGISTER_OFFSET) as *mut u32;
    core::ptr::write_volatile(ptr, value);
    let val = core::ptr::read_volatile(ptr as *const u32);
}
// In real drivers, prefer iowrite32/ioread32 from the kernel crate.
```

### IOMMU and DMA Masks

Before performing DMA, the driver must declare the maximum DMA address the device can handle:

```c
// C: Set the device's DMA mask to 64-bit addresses
dma_set_mask_and_coherent(dev, DMA_BIT_MASK(64));
```

The IOMMU translates device-visible DMA addresses to physical memory addresses, providing:

- **Address translation**: Devices do not see physical memory addresses directly.
- **Protection**: A buggy device cannot corrupt memory outside its allowed region.
- **Large address spaces**: Even 32-bit devices can address >4 GiB with IOMMU remapping.

### Common DMA Bugs (and How Rust Prevents Them)

| Bug | C | Rust mitigation |
|-----|---|-----------------|
| Use-after-free | CPU accesses buffer while device writes | `CoherentDmaBuffer` owned; cannot be freed while borrowed |
| Double-map | Same buffer mapped twice | Ownership prevents aliasing |
| Direction mismatch | `TO_DEVICE` buffer read by CPU without sync | Direction encoded as const generic parameter |
| Missing sync call | Stale CPU cache after device write | `sync_for_cpu()` must be called before dereferencing |
| MMIO without volatile | Compiler removes "dead" register reads | `read_volatile` / `write_volatile` required |

---

## 4. Interrupt Handler Patterns in Rust

**Code snippet**: [`research/snippets/drivers/interrupt_handler.rs`](../research/snippets/drivers/interrupt_handler.rs)

### Interrupt Handler Overview

### IRQ Context Constraints

| Operation | IRQ Context | Process Context |
|-----------|-------------|-----------------|
| Sleep / schedule | ❌ NEVER | ✅ |
| `mutex_lock()` | ❌ NEVER | ✅ |
| `kmalloc(GFP_KERNEL)` | ❌ NEVER | ✅ |
| `kmalloc(GFP_ATOMIC)` | ✅ (may fail) | ✅ |
| Spinlock | ✅ | ✅ |
| Atomics | ✅ | ✅ |
| `copy_to_user()` | ❌ NEVER | ✅ |

### Return Values

Every IRQ handler must return one of:

```rust
IrqReturn::None     // Not our interrupt (shared IRQ line)
IrqReturn::Handled  // Interrupt handled; no further action
IrqReturn::WakeThread  // Handled; also wake the threaded handler
```

### Pattern 1 — Simple Handler

```rust
// Called in hard-IRQ context
unsafe extern "C" fn my_irq_handler(irq: u32, data: *mut c_void) -> IrqReturn {
    let dev = &*(data as *const MyDeviceData);

    // 1. Check if interrupt is from our device
    let status = ioread32(dev.base + STATUS_REG);
    if status & IRQ_PENDING == 0 {
        return IrqReturn::None;
    }

    // 2. Acknowledge the interrupt (clear IRQ at hardware level)
    iowrite32(dev.base + ACK_REG, status & IRQ_PENDING);

    // 3. Record the event (atomic — no locks needed)
    dev.irq_count.fetch_add(1, Ordering::Relaxed);

    IrqReturn::Handled
}
```

### Pattern 2 — Top-Half / Bottom-Half Split

For handlers that need to do non-trivial processing, split the work:

```rust
// Top half: minimal work in IRQ context
fn top_half(&self) -> IrqReturn {
    self.irq_count.fetch_add(1, Ordering::Relaxed);
    schedule_work(&self.work);   // Defer to workqueue
    IrqReturn::Handled
}

// Bottom half: runs in process context (can sleep)
fn bottom_half(&self) {
    // Process DMA buffers, update statistics, notify userspace...
}
```

### Pattern 3 — Threaded IRQ Handler

The modern alternative to explicit workqueues:

```rust
// Register with request_threaded_irq(irq, primary, thread, flags, name, data)

// Primary handler — hard-IRQ context, must be fast
fn primary_handler(&self) -> IrqReturn {
    // Mask the IRQ at hardware level
    iowrite32(self.base + MASK_REG, 0);
    IrqReturn::WakeThread  // Ask kernel to run thread_handler
}

// Thread handler — process context, can sleep
fn thread_handler(&self) -> IrqReturn {
    // Full processing here
    // Re-enable the IRQ when done
    iowrite32(self.base + MASK_REG, IRQ_ENABLE);
    IrqReturn::Handled
}
```

### Accessing Device Data in a Handler

The `dev_id` pointer passed to `request_irq` must:

1. Point to data that remains valid until `free_irq` is called.
2. Use atomics or spinlocks for all shared state.
3. Never use `Mutex` (it can sleep).

```rust
pub struct DeviceData {
    rx_count: AtomicU32,   // ✅ Safe in IRQ context
    // mutex: Mutex<State>,  // ❌ Cannot use in IRQ context
}
```

---

## 5. Error Propagation Mechanisms

**Code snippet**: [`research/snippets/error/error_propagation.rs`](../research/snippets/error/error_propagation.rs)

### Error Propagation Overview

Rust replaces C's ad-hoc error conventions (negative errno, NULL pointers, global `errno`) with
a single, consistent `Result<T, Error>` type. The kernel crate's `Error` type wraps a Linux errno
integer, so all existing error codes are preserved.

### The `?` Operator

The `?` operator is the key ergonomic improvement over C's error handling:

```c
// C — must check every return value manually
int ret = step_one();
if (ret < 0) return ret;

ret = step_two();
if (ret < 0) return ret;
```

```rust
// Rust — ? propagates errors automatically
fn init() -> Result<()> {
    step_one()?;
    step_two()?;
    Ok(())
}
```

### Converting C Return Codes

Most kernel C functions return `0` on success and a negative errno on failure:

```rust
// Wrap any C function that returns an int
fn to_result(ret: i32) -> Result<()> {
    if ret >= 0 { Ok(()) } else { Err(Error::from_errno(-ret)) }
}

// Usage
to_result(unsafe { bindings::some_c_function(arg) })?;
```

### Enriching Errors with Context

Use `.map_err` to transform or log errors before propagating:

```rust
fn open_device(name: &str) -> Result<Device> {
    find_device(name).map_err(|e| {
        pr_err!("Could not find device '{}': {:?}\n", name, e);
        ENODEV
    })
}
```

### `Option` ↔ `Result` Conversions

```rust
// Option::ok_or — convert None to Err
let idx = slice.iter().position(|&x| x == target)
    .ok_or(ENODEV)?;

// Result::ok — convert Err to None (drop the error)
let maybe = fallible_operation().ok();
```

### RAII for Cleanup on Error

Rust's `Drop` trait replaces C's `goto cleanup` pattern:

```rust
fn driver_init() -> Result<DriverState> {
    let buf   = DmaBuffer::alloc(SIZE)?;    // allocated
    let irq   = register_irq(IRQ_NUM)?;    // registered
    let timer = start_timer(INTERVAL)?;    // started
    // If any step fails, all previous resources are freed by Drop.
    // No goto/cleanup needed.
    Ok(DriverState { buf, irq, timer })
}
```

### Error Handling Quick Reference

| Pattern | Use case |
|---------|----------|
| `expr?` | Propagate error to caller immediately |
| `.map_err(\|e\| new_err)` | Transform or log error |
| `.ok_or(err)` | `Option::None` → `Err(err)` |
| `.unwrap_or(default)` | Fallback value (avoid in kernel code) |
| `match result { Ok(v) => ..., Err(e) => ... }` | Handle specific errors differently |
| `if let Err(e) = f() { pr_warn!(...); }` | Log and continue |

> **Warning**: Never use `.unwrap()` in production kernel code. It panics on `Err`, which
> translates to a kernel `BUG()` — unacceptable in a production driver.

---

## Further Reading

- [Kernel Rust API Documentation](https://rust.docs.kernel.org/)
- [Rust for Linux Samples](https://github.com/Rust-for-Linux/linux/tree/rust/samples/rust)
- [DMA API Howto](https://docs.kernel.org/core-api/dma-api-howto.html)
- [Writing an IRQ handler](https://docs.kernel.org/core-api/genericirq.html)
- [Memory allocation guide](https://docs.kernel.org/core-api/memory-allocation.html)
- [RCU concepts](https://docs.kernel.org/RCU/whatisRCU.html)
- [Lock-free programming (Preshing)](https://preshing.com/20120612/an-introduction-to-lock-free-programming/)
- [`research/snippets/`](../research/snippets/) — All code examples in this project
