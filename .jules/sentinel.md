## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2025-04-16 - [Fix SSRF vulnerability in MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF) in `MarketStream::new` (`src/streaming.rs`) where the WebSocket URL passed by the caller wasn't validated, allowing connections to arbitrary servers or internal resources.
**Learning:** Even when building a library for external API fetching, accepting raw user-supplied URLs for data streaming endpoints introduces significant risks if not strictly constrained to the expected external domains.
**Prevention:** Implement strict URL parsing, scheme validation (e.g. `ws` or `wss`), and enforce a whitelist of trusted target domains (`nseindia.com`, `mcxindia.com`) to ensure that users cannot abuse the tool to scan or attack internal network services.
