# Technical Overview

- Kbuild changes to support Rust (`CONFIG_RUST`, rustc flags).
- Interop with C (FFI boundaries, `bindgen` vs. handwritten bindings).
- Safety model inside kernel context (borrow rules, lifetimes vs. kernel refs).
- Subsystems and example drivers implemented or prototyped in Rust.
