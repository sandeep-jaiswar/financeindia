## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2025-02-28 - [Fix SSRF vulnerability in MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF) and scheme bypass in `MarketStream`. The WebSocket client accepted any URL and any host (including `http`/`https` instead of just `ws`/`wss`) to establish market data streams, exposing the application to SSRF attacks if external users can influence the connection URL.
**Learning:** WebSocket streaming clients must validate both the protocol scheme and the target host strictly. Accepting arbitrary URLs for internal streaming components is a common vector for SSRF and accessing internal services or unapproved domains.
**Prevention:** Enforce strict URL scheme validation (only `ws`/`wss`) and use a whitelist of trusted domains (e.g., `*.nseindia.com`, `*.mcxindia.com`) for all externally provided connection URLs. Always parse the URL safely to extract and check the host and scheme before making requests.
