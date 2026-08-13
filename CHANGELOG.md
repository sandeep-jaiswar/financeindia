# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/sandeep-jaiswar/financeindia/compare/v0.1.0...v0.2.0) (2026-08-13)


### Features

* add endpoints for corporate, derivatives, indices, macro and sib ([#6](https://github.com/sandeep-jaiswar/financeindia/issues/6)) ([47093f8](https://github.com/sandeep-jaiswar/financeindia/commit/47093f8724084c383ec02f91daa225bc48ad55ab))
* Add functionality for fetching corporate financial results meta… ([#4](https://github.com/sandeep-jaiswar/financeindia/issues/4)) ([861dce1](https://github.com/sandeep-jaiswar/financeindia/commit/861dce12ccc6b6ce8b900328c860ee5da0a3b42c))
* Add new capital market data endpoints, improve session management with refresh throttling, and introduce comprehensive test scripts. ([0922298](https://github.com/sandeep-jaiswar/financeindia/commit/0922298758dcf0440193f63b23420a8c9bbeaa20))
* add new endpoints for surveillance, derivatives, SLB, and corporate insider trades ([#8](https://github.com/sandeep-jaiswar/financeindia/issues/8)) ([f54242e](https://github.com/sandeep-jaiswar/financeindia/commit/f54242e3449a3c05d57a620d1994f05a159b6a59))
* **ci:** added few env ([43fd8ee](https://github.com/sandeep-jaiswar/financeindia/commit/43fd8ee9d52808cb50f37a8411b23b7fc152a529))
* **ci:** replaced ci.yml with template ci ([c93116f](https://github.com/sandeep-jaiswar/financeindia/commit/c93116faf330d5141c66acc86cc9cd2584898ecd))
* **ci:** replaced ci.yml with template ci with zig ([11672b9](https://github.com/sandeep-jaiswar/financeindia/commit/11672b90f9999a44de9dbf2f4966cc6719789657))
* Expand PyPI publishing workflow to include musllinux, sdist, and additional Linux/Windows/macOS targets using Zig for cross-compilation and uv for publishing. ([3ace966](https://github.com/sandeep-jaiswar/financeindia/commit/3ace966c571649a7d38a10709445cb241d4a8fc3))
* Initial project release including comprehensive endpoint testing, documentation, and licensing. ([96c4d7c](https://github.com/sandeep-jaiswar/financeindia/commit/96c4d7c64c8842a0c1377f944ccbfcd43ef409a0))
* **lib:** add capital market endpoints ([#2](https://github.com/sandeep-jaiswar/financeindia/issues/2)) ([8fbaa1c](https://github.com/sandeep-jaiswar/financeindia/commit/8fbaa1c3025e0b2bfdc2018963651ca94d775c91))
* release 0.1.0 with semantic versioning ([750d22a](https://github.com/sandeep-jaiswar/financeindia/commit/750d22a89b745cf5471113eee25f6dc3eb346d2f))
* **setup:** intialise rust library ([#1](https://github.com/sandeep-jaiswar/financeindia/issues/1)) ([2f781b4](https://github.com/sandeep-jaiswar/financeindia/commit/2f781b472d75c8e4ae22498a7315eea00215b4aa))
* Update Python wheel build to 3.13, explicitly specify the interpreter, and add ABI compatibility and ARM architecture flags. ([f286150](https://github.com/sandeep-jaiswar/financeindia/commit/f286150d9a6a48d426af0951f0e47d3fae473675))


### Bug Fixes

* [HIGH] Fix Zip Slip path traversal during archive generation ([5fccdde](https://github.com/sandeep-jaiswar/financeindia/commit/5fccddec133f39ed9ccd3c9a3a23c710f5d773a6))
* Add `--skip-existing` flag to `uv publish` command. ([cbc8c6a](https://github.com/sandeep-jaiswar/financeindia/commit/cbc8c6a936cd1fa56f6b7af04b5e4dd4d87f5722))
* better error handling ([51656b5](https://github.com/sandeep-jaiswar/financeindia/commit/51656b58e18dc3a332795c09faaf107fc9f23f2b))
* refactor HTTP client and error handling ([#122](https://github.com/sandeep-jaiswar/financeindia/issues/122)) ([136720e](https://github.com/sandeep-jaiswar/financeindia/commit/136720e89c9781ea00b84b007b95978823a66f72))

## [Unreleased]

## [0.1.0] - 2026-08-14

This is the first stable release. Previous `0.1.0-alpha.x` pre-releases are
rolled into this version.

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
- `get_equity_quote`: NSE deprecated the `quote-equity` endpoint (Akamai-blocked); switched to the NextApi `GetQuoteApi` client. The quote is unwrapped from `equityResponse[0]`; live price now lives under `tradeInfo.lastPrice`.
- `get_mcx_bhavcopy`: reqwest is TLS-fingerprinted and blocked by MCX's Akamai WAF. Reimplemented in Python via the optional `curl_cffi` package against the `GetDateWiseBhavCopy` endpoint; returns a parsed list of row dicts. Install with `pip install financeindia[mcx]`.
- `get_option_chain`: use `option-chain-contract-info` + `option-chain-v3` endpoints.
- `bhav_copy_derivatives`: corrected segment archive paths (`/content/fo`, `/content/com`, `/archives/cd/bhav`).
- `get_currency_bhavcopy` / `get_nse_commodities_bhavcopy`: switched to UDiFF bhavcopy zips.
- `get_live_currency_market` / `get_live_commodities_market`: switched to `liveCurrency-derivatives` / `commodity-futures` endpoints.
- Improved session management with thread-safe caching (15-min TTL).
- Optimized date parsing for various Indian date formats.
- Robust error handling and diagnostics for network/HTTP failures.
- Added comprehensive Python docstrings for all methods.
- Removed the dead `validation_error` helper and a stray `common.rs.orig` backup.

[Unreleased]: https://github.com/sandeep-jaiswar/financeindia/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sandeep-jaiswar/financeindia/releases/tag/v0.1.0
