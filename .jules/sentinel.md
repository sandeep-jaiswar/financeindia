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
## 2026-05-06 - [Fix SSRF via Open Redirect in HTTP Client]
**Vulnerability:** The HTTP client configured in `src/common.rs` via `reqwest::ClientBuilder::new().build()` implicitly followed redirects to any domain by default. This allowed a Server-Side Request Forgery (SSRF) vulnerability if an attacker could control an initially trusted domain that then redirected to an arbitrary internal or external host.
**Learning:** By default, `reqwest::Client` follows redirects across any domains. When making outbound requests on behalf of a user, this open redirect behavior bypasses initial URL validations, potentially exposing internal services or causing the server to participate in malicious attacks.
**Prevention:** Always configure a custom `reqwest::redirect::Policy` that explicitly validates and restricts redirect target hosts to a specific whitelist (e.g., `*.nseindia.com`, `*.mcxindia.com`). Additionally, always enforce a maximum redirect depth manually in the custom policy to prevent infinite redirect loops.
