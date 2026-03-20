# Antarcticite (Browser Profile Router)

Antarcticite is a cross-platform background daemon and CLI utility that acts as your operating system's default browser. Instead of opening all links in a single browser, it intelligently routes HTTP/HTTPS links to specific browser profiles based on configured domain rules. It also features a companion browser extension to handle URL resolution for security wrappers like Mimecast.

## Features

- **Rule-based Routing**: Open links matching specific domains or regex patterns in a designated browser and profile.
- **Default Fallback**: Automatically fall back to your normal default browser/profile for any unconfigured domains.
- **Redirect Resolution**: Waits for security wrappers (like Mimecast) to resolve via the companion browser extension before routing the final destination URL.
- **System Tray**: Runs silently in the background with a system tray icon for easy configuration access.
- **Cross-Platform**: Designed for macOS, Windows, and Linux.

## Setup Configuration

1. Create a configuration file at your OS's standard config directory. For example, on macOS/Linux it typically goes to `~/.config/antarcticite/config.toml`.

See [`config.example.toml`](./config.example.toml) for a comprehensive list of examples, including different browsers (Chrome, Edge, Firefox) across all supported operating systems (macOS, Linux, Windows), and various `match_domain` / `match_pattern` rules.

### Finding Browser Identifiers/Binary Names

> [!NOTE]
> The `browser` field requires the application's executable name or its bundle identifier depending on your OS.

- **macOS**: Use the Application Bundle Identifier (e.g., `com.google.chrome`, `com.microsoft.edgemac`, `org.mozilla.firefox`, `company.thebrowser.Browser`). You can find this by running `osascript -e 'id of app "Google Chrome"'` in the terminal.
- **Linux**: Use the `.desktop` file name or the executable name in your PATH (e.g., `google-chrome`, `microsoft-edge-stable`, `firefox`).
- **Windows**: Use the executable name (e.g., `chrome.exe`, `msedge.exe`, `firefox.exe`).

### Finding Profile Directory Names

> [!TIP]
> Browsers usually store profiles in numbered directories (e.g., `Profile 1`, `Profile 2`) rather than the display name you give them. In Chrome/Edge, type `chrome://version` or `edge://version` in the URL bar and look at the "Profile Path" to find the exact folder name.

## Running the App

Run the background routing daemon (with system tray):

```bash
cargo run --release -- daemon
```

## Registering as Default Browser

> [!IMPORTANT]
> Run the installation command to register the application as the default OS handler for HTTP/HTTPS:

```bash
cargo run --release -- install
```

## Installing the Extension

1. Open your default browser.
2. Navigate to your extensions page (e.g., `chrome://extensions` or `edge://extensions`).
3. Enable "Developer mode".
4. Select "Load unpacked" and choose the `extension/` directory from the source code.
5. Make sure the Native Messaging manifest (`com.antarcticite.router.json`) is properly installed in the OS-specific native messaging hosts directory pointing to the built Rust binary (`antarcticite`).

> [!WARNING]
> The extension cannot communicate with the daemon without a correctly configured Native Messaging manifest.

- macOS Chrome: `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/`
- macOS Edge: `~/Library/Application Support/Microsoft Edge/NativeMessagingHosts/`
- Linux Chrome: `~/.config/google-chrome/NativeMessagingHosts/`
- Linux Edge: `~/.config/microsoft-edge/NativeMessagingHosts/`
- Windows Chrome/Edge: Requires a Registry key entry (e.g., `HKCU\Software\Google\Chrome\NativeMessagingHosts\com.antarcticite.router`).

For example, `com.antarcticite.router.json`:
```json
{
  "name": "com.antarcticite.router",
  "description": "Antarcticite Browser Profile Router",
  "path": "/path/to/your/built/antarcticite",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://<YOUR_EXTENSION_ID>/"
  ]
}
```
