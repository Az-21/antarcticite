# Feature Specification: Browser Profile Router

**Feature Branch**: `001-browser-profile-router`  
**Created**: Friday, 20 March 2026  
**Status**: Draft  
**Input**: User description: "Help me create an app which opens specific domains and hardlinks in specific browser with specific profile which would work on macOS, Linux, and Windows. I work with lots of clients and I create browser profiles to keep things separated between clients. say i click on www.clientX.com/<some-complex-link>, then the appropriate profile should handle that link. Edge case: my company uses mimecast which performs a check in the default company profile then resolves the link (it will always have a specific, non-client domain). in this case, the default browser should let the link resolve and the background activity should "see" the resolved URL and open it appropriately in the configured browser"

## Clarifications

### Session 2026-03-20

- Q: How should the app capture the resolved URL after the Mimecast security check completes? → A: Browser Extension
- Q: How should the app communicate errors to the user (e.g., missing profile, parsing error in TOML configuration)? → A: OS Notifications
- Q: How should the user interact with the running application to check its status or access its configuration? → A: System Tray Icon
- Q: How should the system log its routing decisions to help users debug when a link opens in an unexpected profile? → A: Rolling Log File

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic URL Routing (Priority: P1)

As a user working with multiple clients, I want links to specific client domains to open in their designated browser profiles, so my client sessions remain separated.

**Why this priority**: Core functionality of the feature. Without this, the application provides no primary value.

**Independent Test**: Can be fully tested by configuring a rule for a test domain to open in a specific browser profile, clicking a link to that domain, and verifying it opens in the correct profile.

**Acceptance Scenarios**:

1. **Given** a configured routing rule mapping `www.clientX.com` to Chrome Profile A, **When** the user clicks a link to `https://www.clientX.com/dashboard`, **Then** the URL opens in Chrome using Profile A.
2. **Given** a configured routing rule mapping `*.clientY.com` to Firefox Profile B, **When** the user clicks a link to `https://app.clientY.com`, **Then** the URL opens in Firefox using Profile B.

---

### User Story 2 - Handling Unknown URLs (Priority: P2)

As a user, I want URLs that don't match any specific client rules to open in my default browser and profile, so my regular browsing is unaffected.

**Why this priority**: Ensures the app safely falls back and doesn't break normal workflows for non-client URLs.

**Independent Test**: Can be fully tested by clicking a link that has no matching rule and verifying it opens in the default configured browser.

**Acceptance Scenarios**:

1. **Given** a set of client-specific rules and a configured default fallback, **When** the user clicks a link to `https://www.google.com` (which has no specific rule), **Then** the URL opens in the default browser's standard profile.

---

### User Story 3 - Mimecast / Redirect Resolution (Priority: P3)

As an employee using corporate security tools like Mimecast, I want the app to allow the security check to complete and then route the final resolved URL to the correct client profile, so security links don't break my workflow.

**Why this priority**: Solves a specific pain point (edge case) for enterprise users handling wrapped URLs, improving overall reliability.

**Independent Test**: Can be fully tested by clicking a known Mimecast rewritten link, verifying the security check happens, and checking that the final resolved URL is subsequently routed to the appropriate target profile.

**Acceptance Scenarios**:

1. **Given** a Mimecast URL that ultimately resolves to `www.clientX.com`, **When** the user clicks the Mimecast URL, **Then** the default browser opens it for the security check, the system observes the redirect to `www.clientX.com`, and finally opens `www.clientX.com` in Chrome Profile A.

### Edge Cases

- What happens when a configured target browser or profile is deleted or missing from the system? -> The system will fallback to the default browser profile and show an OS notification indicating the missing target.
- How does the system handle rapid, consecutive clicks on the same link? -> The system should debounce rapid clicks to prevent multiple identical browser instances from launching simultaneously.
- How does the system manage recursive or infinite redirects if a URL continually changes? -> The system must enforce a maximum redirect depth and show an OS notification if the limit is exceeded.
- What happens if the Mimecast resolution fails or times out? -> The system will show an OS notification indicating the timeout and stop attempting to resolve the URL.
- What happens if the TOML configuration file contains syntax errors? -> The system will show an OS notification explaining the parsing error and continue using the last known good configuration (or default fallback).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST register itself as the default OS handler for HTTP and HTTPS protocols on macOS, Linux, and Windows.
- **FR-002**: System MUST allow users to define routing rules mapping domains, subdomains, or URL patterns to specific browsers and profiles.
- **FR-003**: System MUST define a default fallback browser and profile for URLs that do not match any routing rules.
- **FR-004**: System MUST observe and handle intermediate redirects (e.g., Mimecast wrappers) by using a companion browser extension installed in the default browser to capture the final resolved URL and send it back to the native app for routing to the target profile.
- **FR-005**: System MUST provide a mechanism to manage, add, edit, and delete routing rules and redirect policies via a simple text configuration file in TOML format.
- **FR-006**: System MUST communicate runtime and configuration errors to the user via native OS notifications.
- **FR-007**: System MUST provide a system tray / menu bar icon to display its running status and provide quick access to open the configuration file or exit the application.
- **FR-008**: System MUST write routing decisions and system events to a rolling plain-text log file located in a standard user application data directory.

### Assumptions

- The user has the necessary OS privileges to set the default browser.
- Browser profiles are already created and managed by the user within their respective browsers.
- The default browser must have the companion extension installed and active for Mimecast redirect resolution to work.
- The system can reliably detect the completion of a Mimecast redirect without requiring deep integration into enterprise authentication systems.

### Key Entities

- **Routing Rule**: Represents a mapping between a URL condition (e.g., domain, regex) and a Target Destination (browser executable, profile identifier).
- **Redirect Policy**: Represents rules for handling intermediate URLs (like Mimecast domains) that require resolution before final routing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully configure at least 10 different domain-to-profile rules and have them route correctly 100% of the time.
- **SC-002**: Link interception and initial routing occurs in under 500 milliseconds (excluding actual browser startup time).
- **SC-003**: The application successfully intercepts and routes HTTP/HTTPS links natively on all three target OS platforms (macOS, Linux, Windows).
- **SC-004**: Mimecast-wrapped URLs correctly resolve and open in the intended target profile within 5 seconds of the initial click.
- **SC-005**: Regular, non-client links open in the default browser fallback without breaking the user's existing workflow.