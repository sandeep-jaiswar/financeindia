## 2024-08-14 - [MEDIUM] Fix WebSocket connection timeout
**Vulnerability:** The `tokio_tungstenite::connect_async` call did not have a timeout configured, leading to potential indefinite hangs (Denial of Service risk) if the external WebSocket server accepted the TCP connection but stalled the TLS/handshake phase.
**Learning:** External API connections natively established without HTTP clients (like `reqwest` which includes default timeouts) may lack built-in timeout logic, especially with underlying TCP/TLS stream futures.
**Prevention:** Always wrap asynchronous connection futures (e.g., `connect_async`) with an explicit runtime timeout (like `tokio::time::timeout`) to ensure defensive programming and predictable error states.
