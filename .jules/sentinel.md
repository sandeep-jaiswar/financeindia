## 2024-05-20 - Path Traversal in BhavArchive
**Vulnerability:** The `BhavArchive.archive_equities` function accepted raw user input for `output_path` and passed it directly to `File::create(path)` without any validation.
**Learning:** This allowed arbitrary file writes anywhere on the system (e.g. `../../etc/passwd` or `/absolute/path.zip`) by exploiting path traversal. The issue existed because the PyO3 wrapper lacked an explicit input sanitization layer before interacting with the host filesystem.
**Prevention:** Validate all file paths received from Python boundaries in Rust before usage. Reject paths containing directory traversal sequences (`..`) or absolute path indicators (`/`, `\`, `:`).
## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.
## 2024-05-25 - [SSRF in MarketStream]
**Vulnerability:** The `MarketStream::new` constructor in `src/streaming.rs` accepted any URL directly without validating its scheme or host. This allowed Server-Side Request Forgery (SSRF) where an attacker could stream internal endpoints or unauthorized hosts via the WebSocket client.
**Learning:** Directly passing user-supplied URLs to network clients (`tokio_tungstenite::connect_async`) without filtering allowed schemes and domains opens up SSRF vectors, even in WebSocket clients.
**Prevention:** Always validate URL schemes and implement an explicit whitelist of trusted domains for outgoing network connections.
## 2024-05-25 - SSRF via Open Redirects
**Vulnerability:** By default, `reqwest::Client` follows HTTP redirects indiscriminately. Since the library fetches data from potentially attacker-controlled inputs (or if the trusted domain is compromised/has an open redirect), the client could be coerced into making requests to unauthorized internal or external services, bypassing the initial domain check.
**Learning:** Initial URL validation is insufficient if the HTTP client can be subsequently redirected to an untrusted domain.
**Prevention:** Always configure a custom redirect policy (`reqwest::redirect::Policy::custom`) that enforces the same domain whitelist rules as the initial request to prevent SSRF via open redirects.
