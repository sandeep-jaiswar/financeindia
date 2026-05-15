## 2024-05-20 - Path Traversal in BhavArchive
**Vulnerability:** The `BhavArchive.archive_equities` function accepted raw user input for `output_path` and passed it directly to `File::create(path)` without any validation.
**Learning:** This allowed arbitrary file writes anywhere on the system (e.g. `../../etc/passwd` or `/absolute/path.zip`) by exploiting path traversal. The issue existed because the PyO3 wrapper lacked an explicit input sanitization layer before interacting with the host filesystem.
**Prevention:** Validate all file paths received from Python boundaries in Rust before usage. Reject paths containing directory traversal sequences (`..`) or absolute path indicators (`/`, `\`, `:`).
## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2024-05-24 - [Fix SSRF in WebSocket Client]
**Vulnerability:** Server-Side Request Forgery (SSRF). In `src/streaming.rs`, the `MarketStream::new` constructor previously accepted arbitrary URLs without validation, allowing a malicious user to initiate WebSocket connections to internal services or untrusted external domains.
**Learning:** Accepting user-controlled URLs for outbound connections without proper scheme and domain validation exposes the system to SSRF vulnerabilities.
**Prevention:** Always validate URLs against a strict whitelist of allowed schemes (e.g., `ws`/`wss`) and trusted domains before initiating outbound connections.
## 2025-04-16 - [Fix SSRF vulnerability in MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF) in `MarketStream::new` (`src/streaming.rs`) where the WebSocket URL passed by the caller wasn't validated, allowing connections to arbitrary servers or internal resources.
**Learning:** Even when building a library for external API fetching, accepting raw user-supplied URLs for data streaming endpoints introduces significant risks if not strictly constrained to the expected external domains.
**Prevention:** Implement strict URL parsing, scheme validation (e.g. `ws` or `wss`), and enforce a whitelist of trusted target domains (`nseindia.com`, `mcxindia.com`) to ensure that users cannot abuse the tool to scan or attack internal network services.
## 2025-05-24 - [Fix SSRF vulnerability in websocket client]
**Vulnerability:** Server-Side Request Forgery (SSRF) in `MarketStream::new` (`src/streaming.rs`). The WebSocket client connected to any URL provided by the user without validating the scheme or host. This could be exploited by an attacker to make the server initiate WebSocket connections to internal services or unintended external endpoints.
**Learning:** Network clients, even WebSockets, that accept arbitrary URLs must explicitly whitelist allowed schemes and hosts to prevent SSRF, particularly when they operate on behalf of a Python application running in potentially sensitive environments.
**Prevention:** Enforce strict URL validation for all network clients. Verify that the protocol scheme is secure and intended (e.g., `ws`, `wss`) and strictly validate the destination host against an explicit whitelist of trusted domains (e.g., `nseindia.com`, `mcxindia.com`).
## 2025-02-28 - [Fix SSRF vulnerability in MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF) and scheme bypass in `MarketStream`. The WebSocket client accepted any URL and any host (including `http`/`https` instead of just `ws`/`wss`) to establish market data streams, exposing the application to SSRF attacks if external users can influence the connection URL.
**Learning:** WebSocket streaming clients must validate both the protocol scheme and the target host strictly. Accepting arbitrary URLs for internal streaming components is a common vector for SSRF and accessing internal services or unapproved domains.
**Prevention:** Enforce strict URL scheme validation (only `ws`/`wss`) and use a whitelist of trusted domains (e.g., `*.nseindia.com`, `*.mcxindia.com`) for all externally provided connection URLs. Always parse the URL safely to extract and check the host and scheme before making requests.
## 2024-05-25 - [SSRF in MarketStream]
**Vulnerability:** The `MarketStream::new` constructor in `src/streaming.rs` accepted any URL directly without validating its scheme or host. This allowed Server-Side Request Forgery (SSRF) where an attacker could stream internal endpoints or unauthorized hosts via the WebSocket client.
**Learning:** Directly passing user-supplied URLs to network clients (`tokio_tungstenite::connect_async`) without filtering allowed schemes and domains opens up SSRF vectors, even in WebSocket clients.
**Prevention:** Always validate URL schemes and implement an explicit whitelist of trusted domains for outgoing network connections.
