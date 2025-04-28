# DOCS_CONVENTIONS

This document defines the standards and conventions for documenting code and project modules in this repository. Follow these guidelines to ensure consistency, readability, and ease of maintenance.

---

## 1. Repository Layout

```
/
├── src/                      ← Rust source code
├── docs/                     ← Markdown-based project documentation
│   ├── DOCS_CONVENTIONS.md   ← This conventions file
│   ├── overview.md           ← High-level architecture and concepts
│   ├── modules/              ← Internal component documentation
│   │   └── module_xyz.md     ← Example module doc
│   └── services/             ← Service-level documentation
│       └── service_logger.md ← Example service doc
└── Cargo.toml                ← Project manifest
```

- **modules/**: for internal, larger components and subsystems.
- **services/**: for reusable services (e.g., logger, wiring, authentication).

---

## 2. Inline (rustdoc) Comments

Use Rust’s built-in documentation comments (`rustdoc`) for all **public** items. Include the following sections using Markdown syntax:

````rust
/// Brief, one-sentence summary of the item.
///
/// # Arguments
/// * `param1` – Description of the first parameter.
/// * `param2` – Description of the second parameter.
///
/// # Returns
/// Description of the return value.
///
/// # Panics
/// Conditions under which the function will panic.
///
/// # Examples
/// ```rust
/// let result = my_fn(42, "example");
/// assert_eq!(result, "expected");
/// ```
pub fn my_fn(param1: i32, param2: &str) -> String {
    // implementation
}
````

At the top of each module (`mod.rs` or `lib.rs`), use inner comments for overview:

```rust
//! High-level overview of the module’s purpose and contents.
//!
//! See:
//! - [`overview.md`](../overview.md)
//! - [`modules/module_xyz.md`](modules/module_xyz.md)
```

---

## 3. Longer-Form Documentation in `docs/`

Organize detailed design docs in separate files under `docs/modules/` and `docs/services/`.

### Folder Structure

```
docs/
├── overview.md
├── modules/
│   └── module_xyz.md
└── services/
    └── service_logger.md
```

### File Content Template

```markdown
# Component Name

Describe the component’s responsibility and context.

## Public API

| Function     | Description                |
| ------------ | -------------------------- |
| `foo()`      | Does foo                   |
| `bar(x: u8)` | Returns `true` if `x > 10` |

## Internals

Explain key algorithms, data structures, invariants, and edge cases.
```

---

## Open Docs

cargo doc --open
