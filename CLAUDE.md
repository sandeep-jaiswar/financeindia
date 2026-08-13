# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`financeindia` is a high-performance Python library written in Rust (PyO3) for fetching Indian financial market data (NSE). It provides both synchronous and asynchronous clients with 42+ API endpoints covering equities, derivatives, indices, corporate actions, and more.

## Development Commands

```bash
# Build and install in development mode
maturin develop

# Run only the offline Python tests
pytest tests/test_stub.py

# Run the full Python test suite (live NSE calls; reachability-dependent)
pytest tests/

# Run Rust unit tests (requires a local Python dev toolchain to link libpython)
cargo test

# Build release wheels
maturin build --release
```

Notes:
- The crate is `cdylib + rlib`. `maturin` builds the wheel with the
  `extension-module` feature (see `pyproject.toml`); `cargo test`/`cargo build`
  run WITHOUT it and link against libpython, so Rust unit tests actually run.
- Minimum supported Rust version is **1.88** (`rust-version` is set in `Cargo.toml`).
- Wheels are built as `cp38-abi3`, matching the Python 3.8+ classifiers.
- The Python `.pyi` stub at `financeindia/financeindia.pyi` is a
  manually-maintained source of truth; `tests/test_stub.py` validates model
  field annotations against `src/models.rs`.

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
- Cookie warm-up: periodic fetch of NSE's all-reports page (15-minute interval); not a rate limiter
- Direct CSV parsing: rows parsed straight into Python objects in Rust
- Concurrent-safe with `RwLock` for session state

## Recent Security Work

The recent commits added Zip Slip vulnerability protection in `archive.rs` - filenames from zip archives are sanitized to prevent path traversal attacks.

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/) with strict enforcement via git hooks:

```bash
# Valid commit examples
git commit -m "feat: add new API endpoint"
git commit -m "fix: resolve authentication issue"
git commit -m "docs: update README"
git commit -m "refactor: simplify error handling"

# Invalid - will be rejected
git commit -m "fixed bug"      # missing type
git commit -m "WIP: something"  # invalid type
```

Allowed types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`

Hooks are in [.husky/](.husky/) and configured via [commitlint.config.js](commitlint.config.js).

## Releasing (Semantic Versioning)

Releases are automated by [release-please](https://github.com/googleapis/release-please)
(`.github/workflows/release-please.yml`) plus the publish pipeline
(`.github/workflows/publish.yml`):

- Version source of truth is `Cargo.toml` (`pyproject.toml` reads it via
  `dynamic = ["version"]`). **Never bump it by hand** — release-please does.
- `.release-please-manifest.json` tracks the current released version.
- Merge conventional-commit PRs to `main`; release-please opens a
  `chore: release X.Y.Z` PR. Merging it creates the `vX.Y.Z` tag + GitHub
  Release, which triggers `publish.yml` to build/publish wheels to PyPI.
- Commit type → bump: `fix:` patch, `feat:` minor, `feat!`/`fix!`/`BREAKING CHANGE:` major.
- To cut the first stable `0.1.0` baseline manually: `git tag v0.1.0 && git push origin v0.1.0`.