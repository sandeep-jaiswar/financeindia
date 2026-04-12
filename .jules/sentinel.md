## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2024-05-24 - [Fix Server-Side Request Forgery (SSRF) in MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF). In `src/streaming.rs`, the `MarketStream::new` constructor accepted an arbitrary URL without validating the scheme or the host, allowing a malicious user to connect to internal services or untrusted endpoints.
**Learning:** All user-provided URLs that the application connects to must be strictly validated to prevent SSRF, especially in libraries where user input might be indirectly controlled by external actors.
**Prevention:** Always validate URL schemes (e.g., `ws`/`wss` for WebSockets, `https` for APIs) and use an allowlist of trusted domains (e.g., `*.nseindia.com`, `*.mcxindia.com`) before establishing connections.
