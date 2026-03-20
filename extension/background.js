const NATIVE_HOST_NAME = "com.antarcticite.router";

console.log("Antarcticite extension background loaded");

// Connect to the native application
let port = null;

function connectToNativeHost() {
  if (port) return;
  port = chrome.runtime.connectNative(NATIVE_HOST_NAME);

  port.onMessage.addListener((msg) => {
    console.log("Received from native host:", msg);
  });

  port.onDisconnect.addListener(() => {
    console.log("Disconnected from native host. Error:", chrome.runtime.lastError?.message);
    port = null;
    // Attempt to reconnect after a delay, or just let the next navigation event trigger a reconnect
  });
}

// We map tabId -> original URL that triggered the redirect flow
const pendingResolutions = new Map();

// When a navigation starts, check if it's hitting a known redirect wrapper.
// In a real app, you would sync this list of domains from the Native App's config.
// For now, we hardcode the Mimecast example from the spec, or allow it to match any pending redirect.
chrome.webNavigation.onBeforeNavigate.addListener((details) => {
  // We only care about top-level main frame navigations
  if (details.frameId !== 0) return;
  
  const url = new URL(details.url);
  // Example: simple check if it looks like a mimecast link
  if (url.hostname.includes("protect-eu.mimecast.com") || url.hostname.includes("mimecast.com")) {
      pendingResolutions.set(details.tabId, details.url);
      console.log(`Tracking redirect for tab ${details.tabId}: ${details.url}`);
  }
});

// When the DOM is fully loaded, or the navigation completes, this might be the final destination.
chrome.webNavigation.onCompleted.addListener((details) => {
  if (details.frameId !== 0) return;

  if (pendingResolutions.has(details.tabId)) {
    const originalUrl = pendingResolutions.get(details.tabId);
    const resolvedUrl = details.url;
    
    // If the resolved URL is different from the original, we consider it resolved
    if (originalUrl !== resolvedUrl) {
      console.log(`Resolved URL for tab ${details.tabId}: ${resolvedUrl}`);
      
      connectToNativeHost();
      
      if (port) {
        port.postMessage({
          type: "ResolvedUrl",
          data: {
            original_url: originalUrl,
            resolved_url: resolvedUrl,
            timestamp_ms: Date.now()
          }
        });
        
        // Clean up
        pendingResolutions.delete(details.tabId);
        
        // Optionally close the tab since we've forwarded it back to the native router
        // chrome.tabs.remove(details.tabId);
      }
    }
  }
});

// Fallback: cleanup if a tab is closed before completing navigation
chrome.tabs.onRemoved.addListener((tabId) => {
  if (pendingResolutions.has(tabId)) {
    pendingResolutions.delete(tabId);
  }
});
