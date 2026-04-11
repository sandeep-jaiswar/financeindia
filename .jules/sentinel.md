## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2023-10-27 - SSRF vulnerability in MarketStream
**Vulnerability:** `MarketStream` allowed connecting to arbitrary WebSocket URLs because it lacked scheme and domain validation in its constructor.
**Learning:** Even though `url::Url::parse` validates format, it doesn't prevent connecting to unauthorized schemes (e.g. `file://` or non-TLS `ws://`) or malicious domains. This is especially problematic for components that accept user input for URLs.
**Prevention:** Always validate URL schemes (e.g. enforcing `wss` or `ws` where appropriate) and implement an allowlist of trusted domains for any outbound network connections.
