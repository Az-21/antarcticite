# Contracts: Browser Extension Native Messaging

The native application and the browser extension communicate via standard streams (stdin/stdout) using UTF-8 JSON messages. Each message is preceded by a 32-bit integer in native byte order containing the message length.

## Message Format (Extension -> Native App)

The extension sends messages to the native app to report the final resolved URL after a redirect (e.g., Mimecast) completes.

### `ResolvedUrl`

Triggered when the extension detects the final destination of a watched URL.

```json
{
  "type": "ResolvedUrl",
  "data": {
    "original_url": "https://protect-eu.mimecast.com/s/...",
    "resolved_url": "https://www.clientX.com/dashboard",
    "timestamp_ms": 1710950400000
  }
}
```

## Message Format (Native App -> Extension)

The native app can optionally send messages back to the extension (e.g., for logging, or to trigger an action, although for this version it is mainly an acknowledgement).

### `Ack`

Sent to acknowledge receipt of a resolved URL.

```json
{
  "type": "Ack",
  "data": {
    "status": "success",
    "message": "URL routed to profile A"
  }
}
```

### `Error`

Sent to communicate errors to the extension (which may be logged in the extension console).

```json
{
  "type": "Error",
  "data": {
    "status": "error",
    "message": "Failed to parse URL or routing rule failed"
  }
}
```
