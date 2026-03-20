# Implementation Plan: Browser Profile Router

**Branch**: `001-browser-profile-router` | **Date**: 2026-03-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/001-browser-profile-router/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Create a cross-platform (macOS, Linux, Windows) application that registers as the default browser and routes HTTP/HTTPS links to specific browser profiles based on configured domain rules. It uses a companion browser extension to handle URL resolution for security wrappers like Mimecast.

## Technical Context

**Language/Version**: Rust (latest stable)
**Primary Dependencies**: tokio, serde, thiserror, anyhow, url, toml, tray-icon, notify-rust, tracing, tracing-appender
**Storage**: Plain-text TOML configuration file, rolling plain-text log file
**Testing**: cargo test (fmt, clippy, check, build, test sequence required)
**Target Platform**: macOS, Linux, Windows
**Project Type**: Desktop Background Service / CLI with System Tray and Native Messaging Host
**Performance Goals**: Link interception and initial routing < 500ms; Mimecast resolution < 5s
**Constraints**: Must register as default OS browser handler. Must handle Native Messaging from the companion extension.
**Scale/Scope**: Local user desktop application with a companion browser extension.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] Does the plan prioritize clarity and readable Rust over cleverness?
- [x] Are files planned to be under 200 lines and modular without catch-all utils?
- [x] Are unit tests with descriptive names/comments planned for every piece of logic?
- [x] Does the plan leverage well-established crates instead of reinventing?
- [x] Are `unwrap()` usages strictly limited to tests?

## Project Structure

### Documentation (this feature)

```text
specs/001-browser-profile-router/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
src/
├── core/
│   ├── config.rs        # TOML configuration parsing and rule management
│   ├── router.rs        # URL routing logic based on rules
│   └── logger.rs        # Rolling log configuration
├── os/
│   ├── default_browser.rs # OS-specific default browser registration
│   ├── notifications.rs # OS-specific notifications via notify-rust
│   └── tray.rs          # System tray icon management
├── extension/
│   └── native_messaging.rs # Stdin/stdout JSON protocol for browser extension
├── cli/
│   └── mod.rs           # Command line argument parsing and routing
└── main.rs              # Entry point

extension/               # Companion browser extension source
├── manifest.json
├── background.js
└── content.js

tests/
├── unit/
└── integration/
```

**Structure Decision**: A single Rust binary project with modular directories for core logic, OS integrations, and extension communication. A separate directory for the companion browser extension source code.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A       |            |                                     |
