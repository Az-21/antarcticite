<!-- 
Sync Impact Report:
- Version change: N/A -> 1.0.0
- List of modified principles:
  - Added: I. Clean & Readable Rust
  - Added: II. Meaningful Documentation
  - Added: III. Modular & Bounded Files
  - Added: IV. Strict Quality & Testing Gates (NON-NEGOTIABLE)
  - Added: V. Ecosystem Integration
- Added sections: 
  - Technology Standards
  - Development Workflow
- Removed sections: N/A
- Templates requiring updates:
  - .specify/templates/plan-template.md (✅ updated)
  - .specify/templates/spec-template.md (✅ updated)
  - .specify/templates/tasks-template.md (✅ updated)
- Follow-up TODOs: None
-->
# antarcticite Constitution

## Core Principles

### I. Clean & Readable Rust
Write clean, readable Rust. Prioritise clarity over cleverness — if a simpler approach is 0.1% slower, use the simpler approach. Prefer concrete implementations over premature abstractions.
*Rationale: Simpler code is easier to maintain and review. Premature abstraction leads to unnecessary complexity.*

### II. Meaningful Documentation
Document the intent of functions (why they exist, what decisions they make, what the invariants are), not a paraphrase of their name.
*Rationale: Comments that duplicate the code name are useless. Intent and invariants provide context that the code alone cannot.*

### III. Modular & Bounded Files
Keep files under 200 lines and separate concerns into distinct module folders. A catch-all `utils.rs` file MUST NOT be used.
*Rationale: Bounded file sizes and explicit modules ensure code remains navigable and focused on a single responsibility.*

### IV. Strict Quality & Testing Gates (NON-NEGOTIABLE)
After every change, run `cargo fmt`, `cargo clippy`, `cargo check`, `cargo build`, and `cargo test` in order, and fix all issues before moving on. Write unit tests for every piece of logic, with test names and comments that describe the behaviour being guarded against regression. Never use `.unwrap()` outside of tests.
*Rationale: Automated checks catch regressions early and enforce formatting consistency. Eliminating unwrap guarantees the software won't crash from unhandled Option/Result.*

### V. Ecosystem Integration
Use well-established crates from the ecosystem rather than reinventing the wheel — reach for tokio, serde, thiserror, anyhow, reqwest, axum, clap, and similar battle-tested libraries where they fit. Always use the latest stable version of any dependency.
*Rationale: Leveraging community-tested crates reduces maintenance burden and increases reliability and security.*

## Technology Standards

All development MUST use Rust. Dependencies MUST be restricted to well-known, battle-tested crates to ensure stability and maintainability.

## Development Workflow

Every change MUST pass the strict quality gates defined in Principle IV. Code MUST be consistently formatted, linted without warnings, and fully tested.

## Governance

Amendments to this constitution require a version bump according to semantic versioning rules. All pull requests and code reviews MUST verify compliance with the core principles, especially the strict quality gates and modularity constraints.

**Version**: 1.0.0 | **Ratified**: 2026-03-20 | **Last Amended**: 2026-03-20
