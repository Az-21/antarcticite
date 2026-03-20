# Implementation Tasks: Browser Profile Router

**Feature**: Browser Profile Router  
**Document Generated**: 2026-03-21

## Phase 1: Setup

- [x] T001 Add primary dependencies (tokio, serde, thiserror, anyhow, url, toml, tracing, tracing-appender) to `Cargo.toml`.
- [x] T002 Create modular directory structure in `src/` (`core/`, `os/`, `cli/`, `extension/`) as defined in the plan.
- [x] T003 Create `src/main.rs` and `src/cli/mod.rs` to set up basic `clap` CLI structure with commands (`daemon`, `install`).
- [x] T004 Create foundational browser extension directory structure (`extension/` with `manifest.json`, `background.js`, `content.js`).

## Phase 2: Foundational 

- [x] T005 Implement `src/core/logger.rs` to configure rolling plain-text logging using `tracing-appender` in the OS app data directory.
- [x] T006 Implement `src/core/config.rs` defining the `Config`, `Rule`, `DefaultFallback`, and `RedirectPolicy` structs matching `data-model.md` and implementing `serde` serialization.
- [x] T007 Implement function to load and parse the TOML configuration file from the standard OS configuration path in `src/core/config.rs`.
- [x] T007a [P] Write unit tests in `src/core/config.rs` for parsing valid and invalid TOML configurations to ensure robust deserialization.
- [x] T007b Implement caching mechanism in `src/core/config.rs` to store the last known good configuration in memory/disk and fallback to it (while triggering an OS notification via `src/os/notifications.rs`) if parsing the updated TOML file fails.

- [x] T008 Implement `src/os/notifications.rs` to wrap `notify-rust` for displaying errors (e.g., config parsing errors) natively.

## Phase 3: User Story 1 - Basic URL Routing

**Goal**: As a user working with multiple clients, I want links to specific client domains to open in their designated browser profiles, so my client sessions remain separated.
**Independent Test**: Configure a rule for a test domain, click a link to that domain, and verify it opens in the correct profile.

- [ ] T009 [US1] Implement `src/core/router.rs` to accept a URL and a `Config` and return the matching `Rule` or `DefaultFallback`.
- [ ] T010 [P] [US1] Write unit tests in `src/core/router.rs` for URL matching logic (exact domain match and regex/pattern match).
- [ ] T011 [US1] Implement OS-specific browser launch logic (handling different browsers and profile flags) triggered by the router in `src/core/router.rs`.
- [ ] T011a [P] [US1] Write unit tests in `src/core/router.rs` (or os module) to verify proper formatting of browser executable arguments and profile flags based on config rules.

- [ ] T012 [P] [US1] Implement CLI logic in `src/cli/mod.rs` to accept a URL as a trailing argument and route it using the loaded config.
- [ ] T013 [US1] Implement Debian/Ubuntu desktop file creation and `xdg-settings` call for Linux default browser registration in `src/os/default_browser.rs`.
- [ ] T014 [P] [US1] Implement macOS default browser registration guidance/automation in `src/os/default_browser.rs`.
- [ ] T015 [P] [US1] Implement Windows default browser registry edits and Settings app launch in `src/os/default_browser.rs`.
- [ ] T016 [US1] Connect the `install` CLI command to the `src/os/default_browser.rs` logic.

## Phase 4: User Story 2 - Handling Unknown URLs

**Goal**: As a user, I want URLs that don't match any specific client rules to open in my default browser and profile, so my regular browsing is unaffected.
**Independent Test**: Click a link that has no matching rule and verify it opens in the default configured browser.

- [ ] T017 [US2] Update `src/core/router.rs` to cleanly fall back to the `DefaultFallback` configuration if no rules match.
- [ ] T018 [P] [US2] Write unit tests for the fallback routing logic in `src/core/router.rs`.
- [ ] T019 [US2] Implement a debounce mechanism in `src/core/router.rs` (or `main.rs`) to prevent rapid, consecutive clicks on the same link from launching multiple browser instances within 500ms.

## Phase 5: User Story 3 - Mimecast / Redirect Resolution

**Goal**: As an employee using corporate security tools like Mimecast, I want the app to allow the security check to complete and then route the final resolved URL to the correct client profile.
**Independent Test**: Click a known Mimecast rewritten link, verify the security check happens, and check that the final resolved URL is subsequently routed to the appropriate target profile.

- [ ] T020 [US3] Implement `src/extension/native_messaging.rs` to handle reading/writing length-prefixed JSON payloads on stdin/stdout.
- [ ] T021 [P] [US3] Define JSON schema structs (`ResolvedUrl`, `Ack`, `Error`) in `src/extension/native_messaging.rs` using `serde` as specified in `contracts/native-messaging.md`.
- [ ] T021a [P] [US3] Write unit tests in `src/extension/native_messaging.rs` for correct JSON serialization and deserialization of the Native Messaging protocol schemas.

- [ ] T022 [US3] Implement the Native Messaging host loop in the `daemon` CLI command (`src/cli/mod.rs`), receiving URLs from the extension and dispatching them to the router.
- [ ] T023 [US3] Implement `extension/manifest.json` with necessary permissions (`webNavigation`, `nativeMessaging`, `storage`) and host matching.
- [ ] T024 [US3] Implement `extension/background.js` to monitor `webNavigation` events for configured redirect domains, capture the final URL, and send a Native Message to `com.antarcticite.router`.
- [ ] T025 [P] [US3] Update `src/core/router.rs` to check if an incoming URL matches a `RedirectPolicy`; if so, open it in the default browser *without* profile flags and rely on the extension to report back.
- [ ] T025a [US3] Implement a maximum redirect depth counter in `src/core/router.rs` for pending resolutions to prevent infinite redirect loops, triggering an OS notification if the limit is exceeded.


## Phase 6: Polish

- [ ] T026 Add `tray-icon` dependency and implement `src/os/tray.rs` to show a system tray icon when running in daemon mode, with a menu to open the config file or quit.
- [ ] T027 Connect the system tray loop alongside the main Native Messaging / daemon loop in `src/main.rs`.
- [ ] T028 Perform full end-to-end integration testing across macOS, Linux, and Windows (or primary development OS first).
- [ ] T029 Add comprehensive README and installation instructions covering the native app, the extension, and Native Messaging manifest installation.

## Dependencies & Execution Order

- **Setup** (Phase 1) is a prerequisite for all other work.
- **Foundational** (Phase 2) builds the core structures (config, logging) needed by all user stories.
- **User Story 1** (Phase 3) requires Phases 1 & 2. It establishes the core routing engine.
- **User Story 2** (Phase 4) builds directly on the router logic from US1.
- **User Story 3** (Phase 5) requires US1 and US2, as it relies on the router to handle both the initial redirect domain (routing it to the default browser) and the final resolved domain.
- **Polish** (Phase 6) can be done at the end.

### Parallel Execution Examples

- While one agent/developer implements the core routing logic (`T009`, `T010`), another can implement the OS-specific browser registration mechanisms (`T013`, `T014`, `T015`).
- The native messaging protocol (`T020`, `T021`) can be built in parallel with the browser extension scripts (`T023`, `T024`).
- Testing and system tray integration (`T026`) can run concurrently once the daemon structure is in place.

## Implementation Strategy

**MVP Scope**: Phase 1, Phase 2, and Phase 3 (User Story 1 & 2 combined logic, excluding debounce). This delivers the core value proposition: intercepting clicks and opening them in the correct profile based on a TOML file.

**Incremental Delivery**:
1. Deliver the MVP (Core routing).
2. Add default fallback and debouncing (Completing US2).
3. Add the browser extension and Native Messaging for Mimecast support (US3).
4. Add the system tray and polish.
