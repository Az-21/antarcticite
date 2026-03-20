# Data Model: Browser Profile Router

## 1. Config Document (TOML)

Represents the user's plain-text TOML configuration file, located at `~/.config/antarcticite/config.toml`.

### Entity: `Config`

- **`default`** (Type: `DefaultFallback`): The fallback browser/profile to use when no routing rules match.
- **`rules`** (Type: `Array<Rule>`): A list of routing rules to apply to intercepted URLs, evaluated in order.
- **`redirect_policies`** (Type: `Array<RedirectPolicy>`): A list of redirect domains (like Mimecast) that the app should wait to resolve before applying routing rules.

### Entity: `DefaultFallback`

- **`browser`** (Type: `String`): Path to the executable, or a known application ID (e.g., `com.google.chrome`, `firefox`).
- **`profile`** (Type: `String`, Optional): The identifier of the profile to use (e.g., `"Profile 1"`, `"Default"`).

### Entity: `Rule`

- **`match_domain`** (Type: `String`, Optional): Exact domain match (e.g., `"www.clientx.com"`).
- **`match_pattern`** (Type: `String`, Optional): Regex or glob pattern for more complex matching (e.g., `".*\.clienty\.com"`).
- **`target_browser`** (Type: `String`): The browser executable or known application ID.
- **`target_profile`** (Type: `String`, Optional): The profile to launch the URL in.

*Validation*: A rule MUST have either `match_domain` or `match_pattern`.

### Entity: `RedirectPolicy`

- **`match_domain`** (Type: `String`): The domain of the redirect wrapper (e.g., `"protect-eu.mimecast.com"`).
- **`timeout_seconds`** (Type: `Integer`, Default: `5`): Maximum time to wait for the final URL from the companion extension.

## 2. In-Memory State

- **Pending Resolutions**: A thread-safe map/cache of active requests (keyed by a session ID or original URL) waiting for the browser extension to report the final resolved URL.
- **Debounce Tracker**: A short-lived cache (e.g., using `moka` or a simple `HashMap` with timestamps) to prevent opening the same URL multiple times within a 500ms window (handling double-clicks).

## 3. Communication Payloads (Native Messaging)

*See `contracts/native-messaging.md` for specific schemas.*
