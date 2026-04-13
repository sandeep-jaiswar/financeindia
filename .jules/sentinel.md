## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2026-04-13 - [Fix SSRF vulnerability in WebSocket MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF) in `MarketStream::new` in `src/streaming.rs`. The WebSocket client accepted any URL from the user and established a connection, allowing a malicious actor to potentially probe internal network addresses or make connections to arbitrary external domains via our application's backend.
**Learning:** WebSocket streaming clients built over generic libraries (like `tokio-tungstenite`) are just as vulnerable to SSRF as HTTP clients. Trusting user-provided URLs in constructor methods without domain/scheme validation bypasses boundary protections.
**Prevention:** Always restrict user-provided URLs to explicit expected schemes (e.g., `ws`/`wss`) and a whitelist of trusted domains or hosts when initialising streaming connections or making HTTP requests on behalf of the user.
