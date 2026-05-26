---
"tauri": patch:bug
---

Recognize the isolation iframe URL as a local origin in `Webview::is_local_url`. The
isolation pattern's URI-scheme handler is registered per-window, not in the manager's
global protocol map, so the post-2.11.1 `.localhost`-suffix check misclassified
`https://{schema}.localhost/` (Windows/Android) and `{schema}://localhost/` (other) as
remote. With the new ACL guard from #15266 in effect, that misclassification blocked the
isolation iframe from receiving IPC plugin internals, breaking isolation-pattern apps on
Windows release builds.
