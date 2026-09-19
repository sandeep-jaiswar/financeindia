# Comparison: financeindia vs Alternatives

This guide helps you choose the right NSE data library for your use case.

## Quick Comparison

| Feature | financeindia | yfinance | pandas_datareader | nsepy |
|---------|--------------|----------|-------------------|-------|
| **Speed** | ⚡ 3-5x faster | Medium | Medium | Slow |
| **NSE Equities** | ✅ 100% | ⚠️ Limited | ⚠️ Limited | ✅ 100% |
| **NSE Derivatives** | ✅ Full (F&O, Currency, Commodities) | ❌ No | ❌ No | ⚠️ Partial |
| **Real-time Quotes** | ✅ Yes | ⚠️ Delayed | ⚠️ Delayed | ⚠️ Delayed |
| **Option Chains** | ✅ Full with Greeks | ❌ No | ❌ No | ✅ Basic |
| **Corporate Actions** | ✅ Dividends, Splits, Bonus, XBRL | ⚠️ Splits only | ❌ No | ❌ No |
| **Async Support** | ✅ Built-in | ❌ No | ❌ No | ❌ No |
| **WebSocket** | ✅ Real-time (beta) | ❌ No | ❌ No | ❌ No |
| **XBRL Financials** | ✅ 500+ data points | ❌ No | ❌ No | ❌ No |
| **Type Safety** | ✅ PyO3 models | ⚠️ Dicts/Series | ⚠️ Series | ⚠️ Partial |
| **Global Markets** | ❌ NSE only | ✅ Global | ✅ Global | ❌ NSE only |
| **Actively Maintained** | ✅ Yes (2026) | ✅ Yes | ⚠️ Slow | ❌ Stale (2022) |
| **License** | 📄 MIT | 📄 Apache 2.0 | 📄 BSD | 📄 MIT |

## Detailed Comparison

### financeindia

**Best For**: Indian quantitative trading, portfolio management, real-time analysis

**Pros**:
- ⚡ **3-5x faster** than alternatives via Rust core
- 📊 **42+ NSE endpoints** with comprehensive coverage
- 🔄 **Async/concurrent** requests for modern Python apps
- 🎯 **Type-safe** PyO3 models (IDE autocomplete, runtime safety)
- 💰 **XBRL financials** for fundamental analysis
- 📈 **Option chains with Greeks** for derivatives trading
- 🔐 **Security-focused** (Zip Slip protection, cookie management)
- 📡 **Real-time** WebSocket support (beta)

**Cons**:
- 🆕 **Alpha stage** (v0.2.3): APIs may change
- 🇮🇳 **NSE-only**: No global market data
- 📦 **Requires Rust 1.88+** to build from source (wheels available)
- ⚠️ **Rate limits**: NSE's Akamai firewall (not our rate limiter)

**When to Use**:
```python
# High-frequency portfolio updates
# Algorithmic trading with derivatives
# Quant research with corporate actions
# Real-time option chain analysis
```

---

### yfinance

**Best For**: Global market data, simple scripts, educational purposes

**Pros**:
- 🌍 **Global coverage**: US, EU, Asia markets
- 📖 **Widely documented**: Thousands of StackOverflow answers
- 🤝 **Large community**: Active GitHub, many tutorials
- ✅ **Stable**: Proven production use

**Cons**:
- 🐢 **Slow**: JSON serialization + pure Python parsing
- ❌ **No async**: Sequential requests block threads
- 🇮🇳 **Poor NSE support**: Limited endpoints, inaccurate data
- 🔧 **Type-unsafe**: Returns dicts/Series (no IDE help)
- 📊 **No derivatives**: F&O, options not supported

**When to Use**:
```python
# Fetching US/EU stock data
# Simple portfolio tracking (non-NSE)
# Educational projects
# Global market research
```

---

### pandas_datareader

**Best For**: Statistical/academic analysis, historical data bulk downloads

**Pros**:
- 📊 **pandas integration**: Seamless DataFrame workflows
- 🏛️ **Multiple data sources**: Yahoo, Fed, World Bank, etc.
- 📚 **Research-friendly**: Built for academic use

**Cons**:
- 🐢 **Very slow**: Double parsing overhead (pandas)
- ❌ **No async**: Network-bound
- 🇮🇳 **Limited NSE**: Only basic Yahoo fallback
- 🔧 **Type-unsafe**: DataFrame columns only

**When to Use**:
```python
# Bulk historical data for backtesting
# Academic research
# Multi-source data aggregation
```

---

### nsepy

**Best For**: NSE-native projects (legacy codebases)

**Pros**:
- 🇮🇳 **NSE-specific**: Good endpoint coverage
- 📖 **Python-native**: No build requirements

**Cons**:
- 🐢 **Slow**: Pure Python, no optimization
- ⏸️ **Unmaintained**: Last commit ~2022
- ❌ **No async**: Sequential only
- 🔧 **Type-unsafe**: No models
- 🪲 **Bugs**: No active maintenance

**When to Use**:
```python
# Legacy codebases already using nsepy
# Simple one-off scripts
# Learning NSE API structure
```

---

## Use Case Matrix

| Use Case | Recommended | Alternative | Avoid |
|----------|------------|-------------|-------|
| **Real-time trading bot** | financeindia | — | nsepy, yfinance |
| **Intraday portfolio tracking** | financeindia | nsepy | yfinance, pandas_datareader |
| **Option chain analysis** | financeindia | — | All others |
| **Corporate actions/XBRL** | financeindia | — | All others |
| **Backtesting (historical)** | financeindia | pandas_datareader | nsepy, yfinance |
| **Global market research** | yfinance | pandas_datareader | — |
| **Quick one-off scripts** | yfinance | financeindia | — |
| **Legacy system migration** | financeindia | — | Stay on nsepy |

## Migration Guide

### From nsepy to financeindia

```python
# OLD (nsepy)
from nsepy import get_history
data = get_history(symbol='RELIANCE', start=date(2026, 1, 1), end=date(2026, 9, 19))
print(data['Close'])

# NEW (financeindia)
import financeindia
client = financeindia.FinanceClient()
data = client.price_volume_data('RELIANCE', '01-01-2026', '19-09-2026')
print(data[0].close_price)  # Type-safe, faster
```

### From yfinance to financeindia

```python
# OLD (yfinance)
import yfinance as yf
reliance = yf.Ticker('RELIANCE.NS')
hist = reliance.history(period='1y')
print(hist['Close'])

# NEW (financeindia)
import financeindia
client = financeindia.FinanceClient()
data = client.price_volume_data('RELIANCE', '01-01-2025', '19-09-2026')
# Richer data: includes volume, open interest, delivery %
```

## Performance Comparison Details

**See [BENCHMARKS.md](BENCHMARKS.md)** for detailed performance metrics and methodology.

### Key Numbers (as of v0.2.3)

| Operation | financeindia | yfinance | Speedup |
|-----------|--------------|----------|---------|
| 7000+ equities list | 850ms | 3200ms | **3.8x** |
| 252 candles (1 year) | 120ms | 450ms | **3.8x** |
| 5000-row option chain | 680ms | N/A | **3.1x** vs nsepy |
| 10 concurrent quotes | 920ms | 4200ms | **4.6x** |

## Choosing the Right Library

### Choose **financeindia** if:
- ✅ You trade on NSE (equities, derivatives, options)
- ✅ You need real-time or low-latency data
- ✅ You value type safety and IDE support
- ✅ You're building trading bots or quant systems
- ✅ You need corporate actions / XBRL financials
- ✅ Performance matters

### Choose **yfinance** if:
- ✅ You only need global market data (non-NSE)
- ✅ You prefer simplicity over speed
- ✅ Your code is already built on yfinance
- ✅ You need broad community support

### Choose **pandas_datareader** if:
- ✅ You're doing academic research
- ✅ You need multiple data sources (Fed, World Bank, etc.)
- ✅ You're comfortable with slow batch operations

### Avoid **nsepy** for:
- ❌ New projects (unmaintained, no async)
- ❌ High-performance systems
- ❌ Modern Python codebases

---

## Questions?

- 📖 Read [EXAMPLES.md](EXAMPLES.md) for code samples
- 🏗️ Check [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- 💬 Open an [Issue](https://github.com/sandeep-jaiswar/financeindia/issues) on GitHub
- 🤝 See [CONTRIBUTING.md](CONTRIBUTING.md) to contribute

---

**Last Updated**: 2026-09-19  
**financeindia Version**: 0.2.3
