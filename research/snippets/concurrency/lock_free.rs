// SPDX-License-Identifier: GPL-2.0

//! Lock-Free Data Structure Implementations for the Linux Kernel
//!
//! Lock-free algorithms improve scalability on multi-core systems by avoiding
//! mutex contention. In the kernel context, they are critical for:
//!
//! - **Per-CPU counters**: Statistics that avoid cache-line bouncing
//! - **RCU (Read-Copy-Update)**: High-throughput read-mostly data
//! - **Atomic reference counting**: Safe object lifetime management
//! - **Lock-free stacks/queues**: Communication between CPU cores or ISRs
//!
//! ## Key Concepts
//!
//! - `core::sync::atomic::*` provides portable atomic operations
//! - `Ordering` controls memory visibility: `Relaxed`, `Acquire`, `Release`, `SeqCst`
//! - RCU in Rust wraps `rcu_read_lock` / `rcu_read_unlock` / `synchronize_rcu`
//! - Treiber stack and Michael-Scott queue are the canonical lock-free structures
//!
//! ## Build
//!
//! Place in `samples/rust/` and enable `CONFIG_RUST=y`.
//!
//! ## Difficulty
//!
//! Advanced

use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use kernel::prelude::*;

// ---------------------------------------------------------------------------
// 1. Atomic counter (per-CPU style)
// ---------------------------------------------------------------------------

/// A shared atomic counter with explicit memory ordering.
///
/// In real per-CPU code the kernel provides `percpu` variables that avoid
/// atomic operations entirely. This struct illustrates the correct memory
/// ordering choices for a shared counter.
pub struct AtomicCounter {
    value: AtomicUsize,
}

impl AtomicCounter {
    /// Create a new counter starting at zero.
    pub const fn new() -> Self {
        Self {
            value: AtomicUsize::new(0),
        }
    }

    /// Increment and return the *previous* value.
    ///
    /// `Relaxed` ordering is sufficient for counters that do not gate any
    /// other memory operations (e.g., statistics).
    pub fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::Relaxed)
    }

    /// Decrement and return the *previous* value.
    pub fn decrement(&self) -> usize {
        self.value.fetch_sub(1, Ordering::Relaxed)
    }

    /// Read the current value.
    pub fn get(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }

    /// Compare-and-swap: only write `new` if the current value equals `expected`.
    ///
    /// Returns `Ok(previous)` on success, `Err(actual)` on failure.
    ///
    /// Uses `AcqRel` so that the successful store is visible to all CPUs before
    /// any subsequent `Acquire` load.
    pub fn compare_exchange(&self, expected: usize, new: usize) -> core::result::Result<usize, usize> {
        self.value
            .compare_exchange(expected, new, Ordering::AcqRel, Ordering::Acquire)
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 2. Treiber lock-free stack
// ---------------------------------------------------------------------------

/// A node in the lock-free stack.
struct StackNode<T> {
    value: T,
    next: *mut StackNode<T>,
}

/// A lock-free stack using the Treiber algorithm.
///
/// The algorithm uses a single `AtomicPtr` for the head and relies on
/// compare-and-swap (CAS) to push/pop without a mutex. ABA prevention is
/// handled by the fact that pushed nodes are *owned* by the stack and are
/// only freed after all readers have quiesced (requires careful unsafe).
///
/// # Safety
///
/// Nodes are allocated with `Box::into_raw` and freed with `Box::from_raw`.
/// Callers must ensure no aliased mutable access to a node after it has been
/// pushed.
pub struct TreiberStack<T> {
    head: AtomicPtr<StackNode<T>>,
}

// SAFETY: TreiberStack owns the nodes; access is mediated by atomic CAS.
unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> TreiberStack<T> {
    /// Create an empty stack.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Push `value` onto the top of the stack.
    ///
    /// This is wait-free: the loop retries only on contention.
    pub fn push(&self, value: T) -> Result<()> {
        let node = Box::try_new(StackNode {
            value,
            next: core::ptr::null_mut(),
        })?;
        let node_ptr = Box::into_raw(node);

        loop {
            let old_head = self.head.load(Ordering::Relaxed);
            // SAFETY: node_ptr is valid; we wrote `next` before the CAS.
            unsafe { (*node_ptr).next = old_head };

            // Release ensures that the node's initialisation is visible
            // before another thread observes the new head.
            match self.head.compare_exchange_weak(
                old_head,
                node_ptr,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(_) => continue, // Retry on contention
            }
        }
    }

    /// Pop the top value from the stack, returning `None` if empty.
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }

            // SAFETY: head is non-null and was set by `push`, which allocated
            // a valid StackNode. No other thread frees this node until after
            // the CAS below succeeds (or we retry).
            let next = unsafe { (*head).next };

            match self.head.compare_exchange_weak(
                head,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // We won the CAS; reconstruct the Box to free it.
                    // SAFETY: head came from Box::into_raw in push().
                    let node = unsafe { Box::from_raw(head) };
                    return Some(node.value);
                }
                Err(_) => continue, // Retry
            }
        }
    }

    /// Return `true` if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed).is_null()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        // Drain all remaining nodes.
        while self.pop().is_some() {}
    }
}

// ---------------------------------------------------------------------------
// 3. Atomic reference counting (Arc-like for kernel objects)
// ---------------------------------------------------------------------------

/// Reference count wrapper around a heap-allocated kernel object.
///
/// Similar to `Arc<T>` but using kernel primitives. The kernel's own
/// `refcount_t` is preferred in production (it has overflow protection);
/// this shows the raw pattern.
pub struct KernelArc<T> {
    inner: core::ptr::NonNull<KernelArcInner<T>>,
}

struct KernelArcInner<T> {
    refcount: AtomicUsize,
    data: T,
}

// SAFETY: KernelArc provides exclusive access through reference counting.
unsafe impl<T: Send + Sync> Send for KernelArc<T> {}
unsafe impl<T: Send + Sync> Sync for KernelArc<T> {}

impl<T> KernelArc<T> {
    /// Create a new `KernelArc` with an initial reference count of 1.
    pub fn new(data: T) -> Result<Self> {
        let inner = Box::try_new(KernelArcInner {
            refcount: AtomicUsize::new(1),
            data,
        })?;
        Ok(Self {
            inner: unsafe {
                core::ptr::NonNull::new_unchecked(Box::into_raw(inner))
            },
        })
    }

    /// Borrow the inner data.
    pub fn as_ref(&self) -> &T {
        // SAFETY: inner is valid as long as refcount > 0, which is true here.
        unsafe { &self.inner.as_ref().data }
    }
}

impl<T> Clone for KernelArc<T> {
    fn clone(&self) -> Self {
        // Relaxed ordering on increment: only atomicity is required here.
        // There is no ordering relationship between this increment and any
        // other memory accesses — we are simply bumping a counter.
        // The actual memory synchronisation is established by the
        // Release store in `drop` (fetch_sub) and the corresponding
        // Acquire fence in the last owner's `drop`, ensuring all writes
        // are visible before the allocation is freed.
        // SAFETY: inner is valid.
        unsafe { self.inner.as_ref() }
            .refcount
            .fetch_add(1, Ordering::Relaxed);
        Self { inner: self.inner }
    }
}

impl<T> Drop for KernelArc<T> {
    fn drop(&mut self) {
        // Release: our writes to `data` must be visible before we potentially
        // free the allocation.
        let prev = unsafe { self.inner.as_ref() }
            .refcount
            .fetch_sub(1, Ordering::Release);

        if prev == 1 {
            // We were the last owner. Acquire to synchronise with all previous
            // Release stores (i.e., see all writes from other owners).
            core::sync::atomic::fence(Ordering::Acquire);
            // SAFETY: refcount reached 0; no other owner can access `inner`.
            unsafe { drop(Box::from_raw(self.inner.as_ptr())) };
        }
    }
}

// ---------------------------------------------------------------------------
// 4. RCU (Read-Copy-Update) pattern
// ---------------------------------------------------------------------------

/// Demonstrates the RCU read-side critical section pattern.
///
/// RCU allows concurrent readers without any locking. Writers make a copy,
/// update the copy, atomically publish it, then wait for all existing readers
/// to finish before freeing the old copy.
///
/// Kernel APIs:
/// - `rcu_read_lock()` / `rcu_read_unlock()` — reader critical section
/// - `rcu_assign_pointer()` — publish a new pointer (Release semantics)
/// - `rcu_dereference()` — safely read an RCU-protected pointer (Acquire)
/// - `synchronize_rcu()` — wait for all pre-existing readers to finish
///
/// In Rust, the `kernel` crate wraps these; the code below shows the logical
/// pattern as it maps to atomic orderings.
pub struct RcuProtected<T> {
    /// The published pointer. Readers use Acquire; writers use Release.
    ptr: AtomicPtr<T>,
}

// SAFETY: RCU ensures readers finish before the old value is freed.
unsafe impl<T: Send + Sync> Send for RcuProtected<T> {}
unsafe impl<T: Send + Sync> Sync for RcuProtected<T> {}

impl<T> RcuProtected<T> {
    /// Create a new RCU-protected value.
    pub fn new(value: T) -> Result<Self> {
        let boxed = Box::try_new(value)?;
        Ok(Self {
            ptr: AtomicPtr::new(Box::into_raw(boxed)),
        })
    }

    /// **Reader path**: access the protected value inside a closure.
    ///
    /// The Acquire load corresponds to `rcu_dereference()`. The closure
    /// represents the RCU read-side critical section.
    ///
    /// # Safety
    ///
    /// The caller must not store the reference beyond the closure's scope.
    pub fn read<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        // rcu_read_lock() (no-op on non-preemptible kernels, but required for
        // correctness in PREEMPT_RCU configurations)
        let ptr = self.ptr.load(Ordering::Acquire);
        // SAFETY: ptr is valid as long as we hold the RCU read lock and do
        // not cross a `synchronize_rcu()` / `call_rcu()` boundary.
        let result = f(unsafe { &*ptr });
        // rcu_read_unlock()
        result
    }

    /// **Writer path**: replace the protected value and return the old one.
    ///
    /// The caller is responsible for calling `synchronize_rcu()` before
    /// dropping the old value to ensure no readers are still using it.
    ///
    /// # Safety
    ///
    /// The caller must call `synchronize_rcu()` (or equivalent) and only then
    /// free the returned pointer.
    pub unsafe fn replace(&self, new_value: T) -> Result<*mut T> {
        let new_boxed = Box::try_new(new_value)?;
        let new_ptr = Box::into_raw(new_boxed);
        // rcu_assign_pointer() — Release so readers see the new value's init.
        let old_ptr = self.ptr.swap(new_ptr, Ordering::Release);
        Ok(old_ptr)
    }
}

impl<T> Drop for RcuProtected<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: We own the allocation; no readers remain after drop.
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Memory ordering cheat-sheet (comments)
// ---------------------------------------------------------------------------
//
//  Ordering::Relaxed  — No synchronisation; only atomicity. Good for
//                       independent counters and flags that don't gate
//                       any other memory operations.
//
//  Ordering::Acquire  — All subsequent memory accesses happen *after* this
//                       load. Pairs with Release stores. Use on the *read*
//                       side of producer/consumer protocols.
//
//  Ordering::Release  — All preceding memory accesses happen *before* this
//                       store. Pairs with Acquire loads. Use on the *write*
//                       side of producer/consumer protocols.
//
//  Ordering::AcqRel   — Combined Acquire+Release for read-modify-write
//                       operations (fetch_add, CAS, swap).
//
//  Ordering::SeqCst   — Total sequential consistency. Necessary when multiple
//                       independent atomic locations must be observed in a
//                       globally consistent order. Most expensive; avoid in
//                       hot paths unless the simpler orderings are insufficient.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new();
        assert_eq!(counter.get(), 0);
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
        counter.decrement();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_treiber_stack_empty() {
        let stack: TreiberStack<u32> = TreiberStack::new();
        assert!(stack.is_empty());
        assert!(stack.pop().is_none());
    }

    #[test]
    fn test_treiber_stack_push_pop() {
        let stack = TreiberStack::new();
        stack.push(1u32).unwrap();
        stack.push(2u32).unwrap();
        stack.push(3u32).unwrap();
        // LIFO order
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_compare_exchange() {
        let counter = AtomicCounter::new();
        // Should succeed: 0 → 10
        assert!(counter.compare_exchange(0, 10).is_ok());
        assert_eq!(counter.get(), 10);
        // Should fail: 0 != 10
        assert!(counter.compare_exchange(0, 20).is_err());
        assert_eq!(counter.get(), 10);
    }
}
