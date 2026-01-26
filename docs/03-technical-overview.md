# Technical Overview

## Kbuild Integration

### Configuration

Rust support is enabled through kernel configuration:

```bash
CONFIG_RUST=y
```

This option becomes available when:
- A compatible Rust compiler is detected (`rustc >= 1.62.0` initially, updated with each kernel version)
- Required tools are present (`bindgen`, `rustfmt`, etc.)
- Target architecture is supported

### Build System Changes

The kernel's Kbuild system was extended to support Rust:

1. **Rust Makefiles**: New rules in `rust/Makefile`
2. **Compiler flags**: Integration of `rustc` flags similar to C compiler flags
3. **Target specifications**: Custom target JSON files for kernel environment
4. **Symbol generation**: Rust symbols compatible with kernel module loading

### Compilation Process

```
Rust source (.rs) → rustc → Object file (.o) → Link with C objects → Kernel module (.ko)
```

Key flags used:
- `--target` - Custom target specification (no_std, kernel features)
- `--emit=obj` - Generate object files compatible with kernel linker
- `-C panic=abort` - No unwinding, panic aborts
- `-C opt-level=2` - Optimization level

## Interop with C

### FFI Boundaries

Rust code interacts with C kernel code through Foreign Function Interface (FFI):

**From Rust to C:**
```rust
extern "C" {
    fn printk(fmt: *const core::ffi::c_char, ...);
}

unsafe {
    printk(b"Hello from Rust\n\0".as_ptr() as *const _);
}
```

**From C to Rust:**
```c
// C header declares Rust function
extern void rust_function(void);

// Call Rust function from C
rust_function();
```

### Bindgen vs. Handwritten Bindings

**Bindgen** - Automatic binding generation:
- Parses C headers and generates Rust bindings
- Used for standard kernel APIs
- Requires careful configuration to handle macros and complex types
- Run at build time

**Handwritten bindings**:
- Used for complex cases where bindgen struggles
- Custom abstractions for unsafe APIs
- Performance-critical code
- Cases requiring special handling

Example bindgen usage:
```rust
// Generated from C headers
use bindings::*;

// Use C types and functions
let ptr = unsafe { kmalloc(size, GFP_KERNEL) };
```

### Safe Abstractions

The Rust-for-Linux project provides safe wrappers:

```rust
// Unsafe C API
extern "C" {
    fn kmalloc(size: usize, flags: u32) -> *mut u8;
    fn kfree(ptr: *mut u8);
}

// Safe Rust wrapper
pub struct KBox<T> {
    ptr: NonNull<T>,
}

impl<T> KBox<T> {
    pub fn new(value: T) -> Result<Self> {
        // Safe allocation wrapper
    }
}

impl<T> Drop for KBox<T> {
    fn drop(&mut self) {
        // Automatic deallocation
    }
}
```

## Safety Model Inside Kernel Context

### Borrow Checker in Kernel Context

Rust's borrow checker ensures:
- **No dangling references**: References always point to valid memory
- **Exclusive mutable access**: Only one mutable reference OR multiple immutable references
- **Lifetime tracking**: Compiler ensures references don't outlive their data

Example:
```rust
fn process_data(data: &mut [u8]) {
    // Compiler ensures:
    // - data is valid
    // - No aliasing mutable references
    // - data lives long enough
}
```

### Borrow Rules vs. Kernel References

**Challenge**: Kernel often uses patterns that violate borrow rules:
- Reference counting (refcount_t)
- RCU (Read-Copy-Update)
- Lock-protected shared state

**Solution**: Use unsafe blocks with safety invariants:

```rust
// Safe wrapper for kernel reference counting
pub struct Ref<T: RefCounted> {
    ptr: NonNull<T>,
    _phantom: PhantomData<T>,
}

impl<T: RefCounted> Clone for Ref<T> {
    fn clone(&self) -> Self {
        // Unsafe: increment refcount
        unsafe { self.ptr.as_ref().inc_ref() };
        Self {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<T: RefCounted> Drop for Ref<T> {
    fn drop(&mut self) {
        // Unsafe: decrement refcount, free if zero
        unsafe {
            if self.ptr.as_ref().dec_ref() {
                // Last reference, deallocate
            }
        }
    }
}
```

### Lifetimes and Kernel Objects

Kernel objects often have complex lifetimes:

```rust
// Device lifetime tied to hardware presence
pub struct Device<'a> {
    // Lifetime parameter ensures device isn't used after removal
    data: &'a DeviceData,
}

// Driver instance
pub struct Driver {
    device: Option<Device<'static>>,
}
```

### Unsafe Code and Safety Invariants

Rust in kernel requires `unsafe` for:
- Calling C functions
- Dereferencing raw pointers
- Implementing certain traits (e.g., `Send`, `Sync`)

**Safety invariants must be documented:**

```rust
// SAFETY: This function is called only from IRQ context
// with interrupts disabled, ensuring no concurrent access.
unsafe fn irq_handler(data: *mut DeviceData) {
    let data = &mut *data;
    data.handle_interrupt();
}
```

## Subsystems and Example Drivers

### Supported Subsystems (as of 2024)

1. **Platform Drivers**
   - Device tree integration
   - Platform device abstraction
   - Example: simple platform drivers

2. **GPIO (General Purpose I/O)**
   - GPIO chip drivers
   - Pin configuration
   - Example: GPIO expander drivers

3. **PHY (Physical Layer)**
   - Network PHY drivers
   - MDIO bus support
   - Example: Ethernet PHY drivers

4. **Character Devices**
   - Device file operations
   - ioctl handling
   - Example: misc devices

5. **Block Devices** (experimental)
   - Block I/O operations
   - Request queue handling

### Example: Simple Rust Driver

```rust
// Sample character device driver
use kernel::prelude::*;

module! {
    type: RustExample,
    name: "rust_example",
    author: "Rust for Linux Contributors",
    description: "Example Rust driver",
    license: "GPL v2",
}

struct RustExample;

impl kernel::Module for RustExample {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust example driver loaded\n");
        Ok(RustExample)
    }
}

impl Drop for RustExample {
    fn drop(&mut self) {
        pr_info!("Rust example driver unloaded\n");
    }
}
```

### Real-World Drivers

As of 2024, several real drivers are being developed:

- **Apple AGX GPU driver** (in progress)
- **Android Binder** (experimental rewrite)
- **Network PHY drivers**
- **Platform drivers** for ARM SoCs

## Architecture Support

Rust is supported on:
- x86_64 (primary development)
- ARM64 (AArch64)
- RISC-V (growing support)
- Other architectures (varying levels of support)

## Performance Characteristics

### Zero-Cost Abstractions

Rust's abstractions compile to the same assembly as hand-written C:

```rust
// Rust
for i in 0..100 {
    array[i] = i * 2;
}

// Compiles to same assembly as C equivalent
```

### Optimization

- LLVM backend provides excellent optimization
- Inlining and devirtualization
- Dead code elimination
- Same optimization levels as C code

### Binary Size

Initial concerns about binary size:
- Rust code can be slightly larger due to monomorphization
- Generic functions generate code for each type
- In practice, size overhead is minimal (1-5%)
- Can be mitigated with careful design

## Development Workflow

1. **Write Rust code** in `drivers/*/` or `rust/kernel/`
2. **Build with** `make LLVM=1`
3. **Test** with QEMU or real hardware
4. **Debug** using standard kernel debugging tools (GDB, KGDB, printk)

## Integration Points

- **Kernel headers**: Accessed via bindgen
- **Kernel macros**: Some translated to Rust, others called through C helpers
- **Module system**: Rust modules work like C modules
- **Debugging**: Compatible with existing kernel debugging infrastructure

## Future Technical Directions

- More subsystem support
- Better macro support
- Improved tooling
- Enhanced abstractions
- Performance optimizations
