# financeindia Roadmap

This document outlines the planned work and future direction for the `financeindia` library. Our goal is to provide the fastest, most reliable Python interface for Indian financial data.

## 🟢 v0.1.x (Current Focus: Production Hardening)
- [x] High-performance Rust core with PyO3 bindings.
- [x] Structured data return (Dicts/Lists) for all 30+ endpoints.
- [x] Exponential backoff retries for NSE rate limiting.
- [x] Connection pooling and `RwLock` for high concurrency.
- [x] Zero-serialization CSV parsing pipeline.

## 🟡 v0.2.0 (Near-Term: Feature Expansion)
- [ ] **Full Async Support**: Expose `asyncio` compatible methods using `reqwest` async core.
- [ ] **Commodities & Currency**: Exhaustive coverage for MCX and NSE Currency segments.
- [ ] **Data Frame Integration**: Optional `polars` integration for direct data loading.


## 🔴 v1.0.0 (Long-Term: Ecosystem & Stability)
- [ ] **Documentation Site**: Dedicated documentation portal with API references and tutorials.
- [ ] **Type Stubs**: Provide `.pyi` files for better IDE autocompletion and type checking.
- [ ] **Historical Data Archive**: Tools for efficient local caching and archival of Bhavcopies.
- [ ] **Stability**: Reaching a stable API for long-term support.

---
*Note: This roadmap is subject to change based on community feedback and NSE technical changes.*
