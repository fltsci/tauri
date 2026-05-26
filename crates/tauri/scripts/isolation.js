// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

if (location.href !== __TEMPLATE_isolation_src__) {
  window.addEventListener('DOMContentLoaded', () => {
    let style = document.createElement('style')
    style.textContent = __TEMPLATE_style__
    document.head.append(style)

    let iframe = document.createElement('iframe')
    iframe.id = '__tauri_isolation__'
    iframe.sandbox.add('allow-scripts')
    // allow-same-origin gives the iframe its own `isolation-{uuid}.localhost`
    // origin instead of a null opaque origin. Without it, WebView2/Chromium
    // refuses to load the iframe's own src URL ("Unsafe attempt to load URL
    // X from frame with URL X") and `window.__TAURI_INTERNALS__` doesn't
    // propagate to the iframe in 2.11.1+. The schema is generated per
    // session and unique-per-process, so the iframe's origin is still
    // distinct from the main app's origin -- the isolation boundary holds.
    iframe.sandbox.add('allow-same-origin')
    iframe.src = __TEMPLATE_isolation_src__
    document.body.append(iframe)
  })
}
