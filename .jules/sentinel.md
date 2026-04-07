## 2024-06-25 - Path Traversal in Zip Archive Filenames
**Vulnerability:** User input was not sanitized before being used as a filename in a zip archive.
**Learning:** The application was vulnerable to path traversal (Zip Slip) because it allowed an attacker to write files outside of the intended directory by supplying a path with `../` sequences or absolute paths (like `/foo`) as the date string.
**Prevention:** Sanitize user input by removing directory separators (`/` and `\`) before using it as a filename when adding files to a zip archive.
