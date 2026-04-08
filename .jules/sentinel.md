## 2025-04-08 - Prevent Zip Slip in BhavArchive
**Vulnerability:** Path traversal (Zip Slip) vulnerability during ZIP file creation
**Learning:** Incorporating unvalidated user-supplied strings (like dates) directly into ZIP archive filenames allows attackers to craft filenames like `../` to manipulate where files are extracted when unzipped, potentially leading to arbitrary file overwrite.
**Prevention:** Always sanitize user-supplied input used in internal archive filenames by replacing OS-specific directory separators ('/' and '\') with safe characters like '_' to ensure files stay within the target extraction directory.
