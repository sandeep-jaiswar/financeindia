# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`financeindia` is a high-performance Python library written in Rust (PyO3) for fetching Indian financial market data (NSE). It provides both synchronous and asynchronous clients with 42+ API endpoints covering equities, derivatives, indices, corporate actions, and more.

## Development Commands

```bash
# Build and install in development mode
maturin develop

# Run only the Python tests
pytest tests/

# Build release wheels
maturin build --release

# Run Rust tests (if any)
cargo test
```

## Architecture

The codebase is organized by API domain:

- **lib.rs** - Main `FinanceClient` struct, PyO3 bindings, and the `fetch_py!` macro for wrapping async functions
- **async_client.rs** - Core async HTTP client with session management and request handling
- **Module files** - `equities.rs`, `derivatives.rs`, `indices.rs`, `corporate.rs`, `slb.rs`, `commodities.rs`, `currency.rs` each implement their respective API endpoints
- **archive.rs** - Handles zip/bhavcopy downloads with Zip Slip protection
- **common.rs** - Shared utilities (date parsing, session refresh intervals)
- **models.rs** - PyO3 data models exposed to Python
- **streaming.rs** - WebSocket support for real-time data

The `fetch_py!` macro bridges async functions to the sync Python API by spawning blocking threads on a shared Tokio runtime.

## Key Patterns

- Methods return typed PyO3 objects (e.g., `MarketStatusResponse`, `EquityInfo`)
- Session caching with automatic refresh (5-minute interval)
- Zero-serialization: CSV data parsed directly into Python objects in Rust
- Concurrent-safe with `RwLock` for session state

## Recent Security Work

The recent commits added Zip Slip vulnerability protection in `archive.rs` - filenames from zip archives are sanitized to prevent path traversal attacks.