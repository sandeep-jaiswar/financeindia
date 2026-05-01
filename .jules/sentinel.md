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
## 2025-05-01 - Prevent SSRF via Open Redirects in HTTP Client

**Vulnerability:** The centralized HTTP client built via `reqwest::ClientBuilder::new()` in `src/common.rs` uses the default redirect policy, which follows up to 10 redirects to any domain. An open redirect vulnerability on initially trusted domains (e.g. `nseindia.com` or `mcxindia.com`) could be leveraged to cause the library to make requests to internal services or malicious domains, potentially leading to Server-Side Request Forgery (SSRF).

**Learning:** When building a client meant to only communicate with specific third-party APIs (like NSE or MCX), relying on default redirect behaviors introduces an unnecessary risk if those APIs have open redirect flaws.

**Prevention:** Explicitly configure a custom `reqwest::redirect::Policy` that enforces a whitelist of trusted domains (and their subdomains) and manually limits the maximum number of redirects.
