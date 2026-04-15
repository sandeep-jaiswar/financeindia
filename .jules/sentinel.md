## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2025-04-15 - [Fix SSRF vulnerability in websocket connection]
**Vulnerability:** Server-Side Request Forgery (SSRF). In `src/streaming.rs`, the `MarketStream::new` constructor accepted arbitrary WebSocket URLs and connected to them without validating the URL scheme or host. This could allow an attacker to make the server establish connections to internal services or malicious external servers.
**Learning:** External or user-controlled URLs must always be validated against a strict whitelist of allowed schemes and domains, especially before establishing long-lived network connections like WebSockets.
**Prevention:** Enforce a strict domain whitelist (e.g., `nseindia.com`, `mcxindia.com`) and restrict allowed URL schemes (`ws`, `wss`) during WebSocket client instantiation to prevent SSRF attacks.
