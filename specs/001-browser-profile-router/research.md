# Research Findings: Browser Profile Router

## 1. OS Default Browser Registration

**Decision**: The application will provide an `install` subcommand to guide or automate the registration process, relying on standard OS mechanisms.

**Rationale**:
- **macOS**: Programmatic registration requires undocumented APIs or prompts the user via `LSSetDefaultHandlerForURLScheme` (which triggers a System Preferences popup). The safest path is generating an `.app` bundle (via `cargo bundle` or `create-dmg`) with `CFBundleURLTypes` mapping `http`/`https` and allowing the user to select the app from System Settings.
- **Windows**: Modern Windows restricts programmatic default browser changes. The app must write specific registry keys (`HKCU\Software\Classes\antarcticite.url` and register in `RegisteredApplications`) and then launch the Windows Settings app to the Default Apps page (`ms-settings:defaultapps`) for the user to make the final selection.
- **Linux**: Using standard XDG utilities, specifically `xdg-settings set default-web-browser antarcticite.desktop`, along with writing an appropriate `.desktop` file to `~/.local/share/applications/`.

**Alternatives considered**: Using third-party crates like `default-browser`, but they are often meant for *reading* the default browser rather than *setting* it, and doing it properly cross-platform requires specific installation flows (registry, `.desktop`, `.app` bundle) which are best handled via custom installation scripts or a dedicated setup step.

## 2. Browser Extension to Native App Communication

**Decision**: Use the Standard WebExtensions Native Messaging API.

**Rationale**: Native Messaging is the official, secure way for browser extensions to communicate with a native application. The native app registers a JSON manifest (e.g., `com.antarcticite.router.json`) with the browser. The browser launches the native app as a subprocess and communicates via standard input/output (stdin/stdout) using JSON messages prefixed with a 32-bit integer length.

**Alternatives considered**:
- **Local HTTP Server / WebSocket**: Running a local web server inside the Rust app and communicating via fetch/WebSocket from the extension. Rejected because it requires keeping a port open, handling CORS, and dealing with potential port conflicts. Native messaging is more secure and directly managed by the browser's lifecycle.

## 3. Cross-Platform System Tray and Notifications

**Decision**: Use the `tray-icon` crate for the system tray and `notify-rust` for notifications.

**Rationale**:
- `tray-icon` (from the Tauri ecosystem) provides a robust, truly cross-platform API for managing tray icons and menus on macOS, Windows, and Linux. It is actively maintained and designed specifically for Rust desktop apps without needing a full GUI framework.
- `notify-rust` is the defacto standard for sending desktop notifications in Rust across all major platforms, integrating natively with macOS UserNotifications, Windows Toast, and Linux DBus notifications.

**Alternatives considered**:
- `ksni` (Linux only tray).
- `tao`/`winit` for windowing which includes tray features, but brings in heavy window management dependencies which aren't strictly necessary for a background router.
