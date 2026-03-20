# Quickstart: Browser Profile Router

## Setup Configuration

1. Create a configuration file at `~/.config/antarcticite/config.toml`.

```toml
[default]
browser = "com.google.chrome"
profile = "Default"

[[rules]]
match_domain = "www.clientX.com"
target_browser = "com.google.chrome"
target_profile = "Profile 1"

[[rules]]
match_pattern = ".*\\.clientY\\.com"
target_browser = "firefox"
target_profile = "Work"

[[redirect_policies]]
match_domain = "protect-eu.mimecast.com"
timeout_seconds = 5
```

## Running the App

Run the background routing daemon (with system tray):

```bash
cargo run --release -- daemon
```

## Registering as Default Browser

Run the installation command to register the application as the default OS handler for HTTP/HTTPS:

```bash
cargo run --release -- install
```

## Installing the Extension

1. Open your default browser.
2. Navigate to your extensions page (e.g., `chrome://extensions`).
3. Enable "Developer mode".
4. Select "Load unpacked" and choose the `extension/` directory from the source code.
5. Make sure the Native Messaging manifest (`com.antarcticite.router.json`) is properly installed in the OS-specific native messaging hosts directory pointing to the Rust binary.
