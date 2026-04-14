## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2024-05-24 - [Fix SSRF in WebSocket Client]
**Vulnerability:** Server-Side Request Forgery (SSRF). In `src/streaming.rs`, the `MarketStream::new` constructor previously accepted arbitrary URLs without validation, allowing a malicious user to initiate WebSocket connections to internal services or untrusted external domains.
**Learning:** Accepting user-controlled URLs for outbound connections without proper scheme and domain validation exposes the system to SSRF vulnerabilities.
**Prevention:** Always validate URLs against a strict whitelist of allowed schemes (e.g., `ws`/`wss`) and trusted domains before initiating outbound connections.
