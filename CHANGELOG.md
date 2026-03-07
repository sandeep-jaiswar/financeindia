# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-07

### Added
- Initial release of the `financeindia` library.
- Comprehensive Capital Market module with 18+ endpoints.
- Support for Equity Lists (All and Nifty 50).
- Historical Price, Volume, and Deliverable data.
- Bulk and Block deals tracking.
- Bhavcopy (UDiFF format) support.
- Live market analysis (Top Gainers/Losers, Most Active).
- Derivatives (Option Chain) support.
- Corporate Actions and Market Holidays.

### Fixed
- Improved session management with thread-safe caching (15-min TTL).
- Optimized date parsing for various Indian date formats.
- Robust error handling and diagnostics for network/HTTP failures.
- Added comprehensive Python docstrings for all methods.
