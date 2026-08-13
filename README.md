# financeindia

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.88+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)](https://www.python.org/)

**financeindia** is a high-performance, lightweight Python library written in Rust for fetching Indian financial market data (NSE) with ease.

> **Data & ToS disclaimer:** `financeindia` scrapes public NSE/MCX pages. NSE and
> MCX can rate-limit or block clients at any time, and the optional MCX path
> impersonates a browser TLS fingerprint to get past Akamai — which may violate
> exchange terms of service. Use this library for personal/research use at your
> own risk, keep request rates low, and do not build commercial redistributors
> of exchange data on top of it.

## Why financeindia?

- **Blazing Fast**: Powered by Rust, providing efficient data parsing and networking.
- **Lightweight**: Minimal dependencies, focus on performance and reliability.
- **Cookie warm-up**: The client periodically touches NSE's `all-reports` page to
  keep Akamai-issued cookies fresh. It is NOT a rate limiter or a guarantee
  against 429s — keep your own request pacing low.
- **Comprehensive**: Access to equity lists, historical data, bhavcopies, indices, and real-time quotes.

## Installation

```bash
pip install financeindia
```

The wheel ships as `cp38-abi3` (one binary for Python 3.8+). Building from
source requires Rust **1.88 or newer** (`edition = "2024"` + let-chains).

## Quickstart

```python
import financeindia

# Initialize the client
client = financeindia.FinanceClient()

# Recommended: initialize the session once
client._initialize_session()

# Get market status (returns a MarketStatusResponse object)
status = client.get_market_status()
print(status.market_state[0].status)

# Fetch historical data for a stock (returns a list of PriceVolumeRow objects)
data = client.price_volume_data("RELIANCE", "01-03-2026", "05-03-2026")
print(data[0].close_price)

# Fetch current quote (returns a dict)
quote = client.get_equity_quote("RELIANCE")
print(quote['tradeInfo']['lastPrice'])

# Fetch Corporate Financial Results
# Returns a list of dicts with filing metadata
results = client.get_financial_results("RELIANCE", "01-01-2025", "07-03-2026", "Annual")

# Parse the XBRL detail for a specific filing
if results:
    xbrl_url = results[0]['xbrl']
    financial_details = client.get_financial_details(xbrl_url)
    print(financial_details) # Highly granular financial data points
```

## Supported Endpoints (42+)

`financeindia` provides exhaustive coverage of NSE data. Methods return typed PyO3 model classes (attribute-accessible objects) and, when applicable, lists of those model instances (e.g., `MarketStatusResponse`, `MarketStatus`, `FiiDiiActivity`, `ASMStock`, `GSMStock`, `Holiday`, `EquityInfo`). See [tests/test_client.py:13](tests/test_client.py#L13) as an example.

### 📊 Market Macro
- `get_market_status()`: Current status of all market segments.
- `get_holidays()`: Trading holiday calendar.
- `get_fii_dii_activity()`: Daily FII and DII trading activity.
- `get_market_turnover()`: Market-wide turnover statistics.
- `get_fii_stats(date)`: Detailed FII statistics (returns raw XLS bytes).

### 🔍 Surveillance & Monitoring
- `get_asm_stocks()`: Additional Surveillance Measure (ASM) stocks (Long & Short term).
- `get_gsm_stocks()`: Graded Surveillance Measure (GSM) list.
- `get_short_ban_stocks()`: Filtered list of stocks under short-term surveillance.

### 📈 Equities
- `get_equity_list()`: All active equities listed on NSE.
- `price_volume_data(symbol, from, to)`: Historical price and volume.
- `deliverable_position_data(symbol, from, to)`: Delivery percentage stats.
- `bhav_copy_equities(date)`: Daily equity bhavcopy.
- `bulk_deal_data(from, to)`: Tracking large institutional bulk deals.
- `block_deals_data(from, to)`: Tracking block deal transactions.
- `short_selling_data(from, to)`: Daily short selling activity.
- `get_52week_high_low(type)`: Stocks hitting new highs or lows.
- `get_top_gainers()` / `get_top_losers()`: Intraday performance leaders.
- `get_most_active(index)`: Most active securities by volume/value.
- `get_advances_declines()`: Market breadth analysis.
- `get_monthly_settlement_stats(year)`: Monthly settlement data.
- `get_equity_quote(symbol)`: Real-time price and order book.

### 📉 Indices
- `get_all_indices()`: Real-time snapshot of all NSE indices.
- `get_index_constituents(index)`: Stocks within a specific index (e.g., 'NIFTY 50').
- `get_index_history(index, from, to)`: Historical index levels.
- `get_index_yield(index, from, to)`: P/E, P/B, and Dividend Yield of indices.
- `get_india_vix_history(from, to)`: Historical volatility index data.
- `get_total_returns_index(index, from, to)`: Total Returns Index (TRI) data.

### ⛓️ Derivatives
- `get_option_chain(symbol, is_index)`: Full real-time option chain with Greeks.
- `bhav_copy_derivatives(date, segment)`: Daily F&O/Currency/Commodity bhavcopy.
- `get_fo_sec_ban()`: Current F&O ban list (JSON).
- `get_fo_ban_list(date)`: Historical F&O ban list (CSV).
- `get_span_margins(date)`: Daily SPAN margin files for risk analysis.
- `get_oi_limits_cli(date)`: Daily Client wise OI Limit (LST).
- `get_participant_volume(date)`: Participant wise trading volumes.

### 🤝 Securities Lending & Borrowing (SLB)
- `get_slb_bhavcopy(date)`: Daily SLB market bhavcopy.
- `get_slb_eligible()`: Real-time list of securities available for SLB.
- `get_slb_open_positions(series)`: Open positions analysis for a specific month.
- `get_slb_series_master()`: Helper to find active SLB series/months.

### 📊 Commodities & MCX
- `get_live_commodities_market()`: NSE commodity futures market watch.
- `get_nse_commodities_bhavcopy(date)`: NSE Commodities bhavcopy.
- `get_mcx_bhavcopy(date, instrument='ALL')`: MCX daily bhavcopy as a list of row dicts (futures & options). MCX's Akamai firewall requires the optional [`curl_cffi`](https://pypi.org/project/curl_cffi/) package — install with `pip install curl_cffi`. Note the TLS-impersonation caveat in the disclaimer above.

### 🏢 Corporate Actions
- `get_corporate_actions()`: Latest dividends, bonuses, splits, etc.
- `get_financial_results(symbol, from, to, period)`: Metadata for financial filings.
- `get_financial_details(xbrl_url)`: Deep-dive into XBRL filings (500+ data points).
- `get_insider_trades(from, to)`: Detailed PIT (Prohibition of Insider Trading) disclosures.

## Performance Optimizations

`financeindia` is built for high-scale quant pipelines:
- **Direct CSV Parsing**: CSV data is parsed into Python objects in Rust, without an intermediate Python-visible string round-trip.
- **Concurrent-Safe**: Uses `RwLock` and optimized connection pooling for multi-threaded usage.
- **Blazing Fast**: Up to 3x faster than traditional JSON-based wrappers.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.