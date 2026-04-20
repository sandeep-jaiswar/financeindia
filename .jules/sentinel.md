## 2024-05-20 - Path Traversal in BhavArchive
**Vulnerability:** The `BhavArchive.archive_equities` function accepted raw user input for `output_path` and passed it directly to `File::create(path)` without any validation.
**Learning:** This allowed arbitrary file writes anywhere on the system (e.g. `../../etc/passwd` or `/absolute/path.zip`) by exploiting path traversal. The issue existed because the PyO3 wrapper lacked an explicit input sanitization layer before interacting with the host filesystem.
**Prevention:** Validate all file paths received from Python boundaries in Rust before usage. Reject paths containing directory traversal sequences (`..`) or absolute path indicators (`/`, `\`, `:`).
## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2025-05-24 - [Fix SSRF vulnerability in websocket client]
**Vulnerability:** Server-Side Request Forgery (SSRF) in `MarketStream::new` (`src/streaming.rs`). The WebSocket client connected to any URL provided by the user without validating the scheme or host. This could be exploited by an attacker to make the server initiate WebSocket connections to internal services or unintended external endpoints.
**Learning:** Network clients, even WebSockets, that accept arbitrary URLs must explicitly whitelist allowed schemes and hosts to prevent SSRF, particularly when they operate on behalf of a Python application running in potentially sensitive environments.
**Prevention:** Enforce strict URL validation for all network clients. Verify that the protocol scheme is secure and intended (e.g., `ws`, `wss`) and strictly validate the destination host against an explicit whitelist of trusted domains (e.g., `nseindia.com`, `mcxindia.com`).
