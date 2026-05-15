## 2024-04-06 - Path Traversal in ZIP Filenames
**Vulnerability:** User-supplied strings (dates) were being directly interpolated into internal ZIP archive filenames (`zip.start_file(format!("bhav_{}.csv", date), options)`) without sanitization in `BhavArchive`. This could allow an attacker to inject `../` sequences in the `date` parameter, potentially writing files outside the intended archive directory upon extraction, leading to a Zip Slip / Path Traversal vulnerability.
**Learning:** Even when incorporating user input into internal strings like filenames within an archive, explicit sanitization is required because the resulting artifact (a ZIP file) carries those potentially malicious paths to the extracting application.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' before using them in internal path generation to prevent Zip Slip and path traversal vulnerabilities.
## 2024-06-25 - Path Traversal in Zip Archive Filenames
**Vulnerability:** User input was not sanitized before being used as a filename in a zip archive.
**Learning:** The application was vulnerable to path traversal (Zip Slip) because it allowed an attacker to write files outside of the intended directory by supplying a path with `../` sequences or absolute paths (like `/foo`) as the date string.
**Prevention:** Sanitize user input by removing directory separators (`/` and `\`) before using it as a filename when adding files to a zip archive.
## 2025-04-08 - Prevent Zip Slip in BhavArchive
**Vulnerability:** Path traversal (Zip Slip) vulnerability during ZIP file creation
**Learning:** Incorporating unvalidated user-supplied strings (like dates) directly into ZIP archive filenames allows attackers to craft filenames like `../` to manipulate where files are extracted when unzipped, potentially leading to arbitrary file overwrite.
**Prevention:** Always sanitize user-supplied input used in internal archive filenames by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' to ensure files stay within the target extraction directory.
## 2025-02-28 - Zip Slip / Path Traversal Risk in Archive Filenames
**Vulnerability:** Path traversal via unsanitized user-supplied strings used directly in ZIP archive filenames (e.g., `zip.start_file(format!("bhav_{}.csv", date), options)` in `src/archive.rs`).
**Learning:** Incorporating external input (like user-requested dates) into paths within an archive can allow attackers to inject path traversal sequences (`../`), potentially causing extracted files to overwrite arbitrary local files (Zip Slip).
**Prevention:** Always sanitize any user-supplied strings intended for filenames by replacing OS-specific directory separators (`/` and `\`) with a safe character like `_` before embedding them into archive structures.
## 2024-05-20 - Path Traversal in BhavArchive
**Vulnerability:** The `BhavArchive.archive_equities` function accepted raw user input for `output_path` and passed it directly to `File::create(path)` without any validation.
**Learning:** This allowed arbitrary file writes anywhere on the system (e.g. `../../etc/passwd` or `/absolute/path.zip`) by exploiting path traversal. The issue existed because the PyO3 wrapper lacked an explicit input sanitization layer before interacting with the host filesystem.
**Prevention:** Validate all file paths received from Python boundaries in Rust before usage. Reject paths containing directory traversal sequences (`..`) or absolute path indicators (`/`, `\`, `:`).
## 2024-05-24 - [Fix path traversal in zip filename]
**Vulnerability:** Path Traversal / Zip Slip. In `src/archive.rs`, user-supplied dates were directly formatted into zip filenames, which would allow a malicious user to supply path traversal characters (`/` or `\`) in the date string and overwrite files outside the intended extraction directory.
**Learning:** Directly interpolating user-controlled strings into file paths inside a zip archive without sanitization is a critical vulnerability vector.
**Prevention:** Always sanitize input by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' when using them to construct internal ZIP archive filenames.

## 2023-10-27 - SSRF vulnerability in MarketStream
**Vulnerability:** `MarketStream` allowed connecting to arbitrary WebSocket URLs because it lacked scheme and domain validation in its constructor.
**Learning:** Even though `url::Url::parse` validates format, it doesn't prevent connecting to unauthorized schemes (e.g. `file://` or non-TLS `ws://`) or malicious domains. This is especially problematic for components that accept user input for URLs.
**Prevention:** Always validate URL schemes (e.g. enforcing `wss` or `ws` where appropriate) and implement an allowlist of trusted domains for any outbound network connections.
## 2024-05-24 - [Fix Server-Side Request Forgery (SSRF) in MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF). In `src/streaming.rs`, the `MarketStream::new` constructor accepted an arbitrary URL without validating the scheme or the host, allowing a malicious user to connect to internal services or untrusted endpoints.
**Learning:** All user-provided URLs that the application connects to must be strictly validated to prevent SSRF, especially in libraries where user input might be indirectly controlled by external actors.
**Prevention:** Always validate URL schemes (e.g., `ws`/`wss` for WebSockets, `https` for APIs) and use an allowlist of trusted domains (e.g., `*.nseindia.com`, `*.mcxindia.com`) before establishing connections.
## 2026-04-13 - [Fix SSRF vulnerability in WebSocket MarketStream]
**Vulnerability:** Server-Side Request Forgery (SSRF) in `MarketStream::new` in `src/streaming.rs`. The WebSocket client accepted any URL from the user and established a connection, allowing a malicious actor to potentially probe internal network addresses or make connections to arbitrary external domains via our application's backend.
**Learning:** WebSocket streaming clients built over generic libraries (like `tokio-tungstenite`) are just as vulnerable to SSRF as HTTP clients. Trusting user-provided URLs in constructor methods without domain/scheme validation bypasses boundary protections.
**Prevention:** Always restrict user-provided URLs to explicit expected schemes (e.g., `ws`/`wss`) and a whitelist of trusted domains or hosts when initialising streaming connections or making HTTP requests on behalf of the user.
## 2025-04-15 - [Fix SSRF vulnerability in websocket connection]
**Vulnerability:** Server-Side Request Forgery (SSRF). In `src/streaming.rs`, the `MarketStream::new` constructor accepted arbitrary WebSocket URLs and connected to them without validating the URL scheme or host. This could allow an attacker to make the server establish connections to internal services or malicious external servers.
**Learning:** External or user-controlled URLs must always be validated against a strict whitelist of allowed schemes and domains, especially before establishing long-lived network connections like WebSockets.
**Prevention:** Enforce a strict domain whitelist (e.g., `nseindia.com`, `mcxindia.com`) and restrict allowed URL schemes (`ws`, `wss`) during WebSocket client instantiation to prevent SSRF attacks.
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
