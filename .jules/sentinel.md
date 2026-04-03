
## 2024-05-24 - Zip Slip / Path Traversal in Archive Filenames
**Vulnerability:** The application was using untrusted user input directly to generate filenames within a ZIP archive. This allows a malicious user to supply path traversal sequences (like `../`) that could result in arbitrary file overwrites when the generated ZIP is subsequently extracted.
**Learning:** Even internal data processing mechanisms like dynamically generating archives need strict sanitization of variables inserted into structural file components. Input should never be implicitly trusted as just a safe value like a simple "date string".
**Prevention:** Always sanitize any dynamic or user-controlled input used as part of a file path, even when generating files (like ZIP contents). In this specific instance, explicitly replacing OS-specific directory separators (`/` and `\`) with a safe character (`_`) mitigates the risk.
