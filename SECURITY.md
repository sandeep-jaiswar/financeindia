# Security Policy

## Overview

`financeindia` is designed with security as a core principle. This document outlines our security practices, known issues, and how to report vulnerabilities.

## Security Measures

### 1. Zip Slip Prevention (Archive Security)

**Issue**: Malicious ZIP files can extract files outside the target directory via path traversal.

**Fix** (v0.2.3+): All archive extraction validates paths:
```rust
// In src/archive.rs
let outpath = target_dir.join(file.name());

// Prevent directory traversal
if !outpath.starts_with(target_dir) {
    return Err("Path traversal attempt detected");
}
```

**Impact**: Safe handling of `bhav_copy_equities()`, `bhav_copy_derivatives()`, etc.

### 2. Memory Safety

**Rust Guarantees**:
- ✅ No buffer overflows (bounds checking enforced)
- ✅ No use-after-free (ownership system)
- ✅ No data races (borrow checker)
- ✅ No integer underflow (checked arithmetic)

### 3. Dependency Management

**Practices**:
- ⚠️ Minimize dependencies (8 core Rust crates)
- 🔄 Regular updates via Dependabot
- 🔐 Pin versions for reproducible builds
- 🚨 Security audits via `cargo audit`

```bash
cargo audit  # Check for known vulnerabilities
```

### 4. Cookie & Session Management

**Features**:
- Cookies stored in-memory (not persisted to disk)
- 15-minute expiration (NSE Akamai requirement)
- Automatic refresh without user interaction

### 5. TLS/HTTPS

**Enforced**:
- All requests to NSE use TLS 1.2+
- Certificate validation enabled
- Supported via `rustls` (Rust-native, audited)

### 6. Input Validation

**Validated**:
- Date formats (DD-MM-YYYY)
- Symbol names (uppercase, max 10 chars)
- Numeric ranges (prices, volumes)
- URL paths (prevent injection)

---

## Supported Versions

| Version | Security Updates |
|---------|------------------|
| 0.2.x   | ✅ Yes           |
| 0.1.x   | ⚠️ Limited       |
| < 0.1   | ❌ No            |

---

## Known Limitations & Disclaimers

### Data Source Warning

**Important**: `financeindia` **scrapes public NSE/MCX web pages**, not official APIs.

**Risks**:
1. **Rate Limiting**: NSE/Akamai may block your IP for excessive requests
   - Keep request rate low (1-2 per second)
   - Cookie warm-up is NOT rate limiting; you must pace requests
   
2. **Data Accuracy**: Web pages may be outdated
   - Validate against NSE's official website
   
3. **API Breakage**: NSE may change page layouts
   - Requires code updates
   
4. **Terms of Service**: NSE's ToS may prohibit scraping
   - Use for personal/research only
   - Do not build commercial redistributors

### MCX (Optional) TLS Fingerprinting

**For MCX data**, the optional `curl_cffi` feature impersonates browser TLS fingerprints:
- May violate exchange terms of service
- Install with: `pip install financeindia[mcx]`
- Use only for personal/authorized projects

---

## Reporting Security Issues

**DO NOT** open public GitHub issues for security vulnerabilities.

### Secure Reporting

**Email**: `jaiswarsandeep119@gmail.com`

**Include**:
- Vulnerability description
- Steps to reproduce
- Potential impact
- Suggested fix (if available)

**Timeline**:
- Response: Within 48 hours
- Fix: Within 7-14 days (depending on severity)
- Disclosure: After patch release

### Severity Levels

| Level | Impact | Timeline |
|-------|--------|----------|
| **Critical** | Code execution, data loss | 2-3 days |
| **High** | Auth bypass, info disclosure | 5-7 days |
| **Medium** | Denial of service | 7-14 days |
| **Low** | Edge cases, docs | 14-30 days |

---

## Security Best Practices for Users

### 1. Validate Input
```python
# ✅ Good: validate before use
user_symbol = input("Enter symbol: ").upper()
if not user_symbol.isalnum():
    raise ValueError("Invalid symbol")
data = client.price_volume_data(user_symbol, "01-01-2026", "19-09-2026")
```

### 2. Handle Errors Gracefully
```python
try:
    data = client.get_equity_quote("RELIANCE")
except Exception as e:
    logger.error(f"Failed: {e}")  # Don't expose to users
```

### 3. Rate Limiting
```python
import time
time.sleep(0.5)  # 500ms between requests
quote = client.get_equity_quote(symbol)
```

### 4. Session Initialization
```python
client = financeindia.FinanceClient()
client._initialize_session()  # Warm up cookies once
```

### 5. Monitor Dependencies
```bash
pip install --upgrade financeindia
pip audit
```

---

## Security Testing

```bash
# Check Rust dependencies
cargo audit

# Run tests
cargo test
pytest tests/

# Check for exposed secrets
git log --all --grep="secret\|password"
```

---

## Security Roadmap

- [ ] Formal security audit (Q4 2026)
- [ ] Fuzzing campaign for archive parsing
- [ ] Built-in rate limiting (advisory)
- [ ] Official NSE API integration

---

## Questions?

- 📧 Security: `jaiswarsandeep119@gmail.com`
- 💬 General: [GitHub Discussions](https://github.com/sandeep-jaiswar/financeindia/discussions)
- 🐛 Bugs: [GitHub Issues](https://github.com/sandeep-jaiswar/financeindia/issues)

---

**Last Updated**: 2026-09-19  
**financeindia Version**: 0.2.3
