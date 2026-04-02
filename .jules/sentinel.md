## 2025-04-03 - Path Traversal in ZIP Archive Creation
**Vulnerability:** A path traversal vulnerability existed in `src/archive.rs` where user-supplied `date` strings were used directly in the `zip.start_file` method. This allowed an attacker to create files outside the intended directory by supplying a date like `../../../../tmp/evil.csv`.
**Learning:** The `zip` crate in Rust does not automatically sanitize file paths within archives. Developers must be aware of how user input influences archive file names.
**Prevention:** Always sanitize user-supplied strings before using them in archive file names by replacing OS-specific directory separators ('/' and '\') with safe characters like '_'.
