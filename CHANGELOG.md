# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- `get_equity_quote`: NSE deprecated the `quote-equity` endpoint (Akamai-blocked); switched to the NextApi `GetQuoteApi` client. The quote is unwrapped from `equityResponse[0]`; live price now lives under `tradeInfo.lastPrice`.
- `get_mcx_bhavcopy`: reqwest is TLS-fingerprinted and blocked by MCX's Akamai WAF. Reimplemented in Python via the optional `curl_cffi` package against the `GetDateWiseBhavCopy` endpoint; returns a parsed list of row dicts. Install with `pip install financeindia[mcx]`.
- `get_option_chain`: use `option-chain-contract-info` + `option-chain-v3` endpoints.
- `bhav_copy_derivatives`: corrected segment archive paths (`/content/fo`, `/content/com`, `/archives/cd/bhav`).
- `get_currency_bhavcopy` / `get_nse_commodities_bhavcopy`: switched to UDiFF bhavcopy zips.
- `get_live_currency_market` / `get_live_commodities_market`: switched to `liveCurrency-derivatives` / `commodity-futures` endpoints.
- Removed the dead `validation_error` helper and a stray `common.rs.orig` backup.

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
