# financeindia Examples

Production-ready examples for common use cases.

## Quick Start

All examples require `financeindia` to be installed:

```bash
pip install financeindia
```

Then run any example:

```bash
python portfolio_dashboard.py
python option_analysis.py
python market_monitor.py
python backtest_simple_ma.py
```

## Examples

### 1. Portfolio Dashboard (`portfolio_dashboard.py`)

Real-time P&L tracking for a stock portfolio.

**Features**:
- Fetch current quotes for multiple stocks
- Calculate unrealized P&L
- Display portfolio composition

**Usage**:
```python
dashboard = PortfolioDashboard()
dashboard.update_prices()
```

**Output**:
```
Symbol     Qty    Buy        Current    P&L          P&L %
------------------------------------------------------
RELIANCE   100    2850.00    3120.00    27000.00     9.47%
INFY       50     3400.00    3650.00    12500.00     7.35%
TCS        25     3800.00    4100.00    7500.00      7.89%
------------------------------------------------------
TOTAL                                   47000.00     8.60%
Portfolio Value: ₹1,247,500.00
```

### 2. Option Chain Analysis (`option_analysis.py`)

Identify high-IV option opportunities for selling.

**Features**:
- Fetch full option chain for NIFTY/BANKNIFTY/stocks
- Filter by implied volatility (IV)
- Display bid-ask spreads

**Usage**:
```python
analyzer = OptionChainAnalyzer("NIFTY")
analyzer.print_report()
```

**Output**:
```
=== High IV Opportunities (NIFTY) ===

Strike   Type  IV        Bid      Ask
----------------------------------------
 18800   CE     32.5%    45.20   46.80
 18900   PE     31.2%    42.10   43.50
 19000   CE     30.8%    38.90   40.10
 ...
```

### 3. Market Monitoring (`market_monitor.py`)

Real-time NSE market health dashboard.

**Features**:
- Market segment status
- FII/DII activity (buy/sell volumes)
- Market breadth (advances vs. declines)
- Top gainers/losers

**Usage**:
```python
monitor = MarketMonitor()
monitor.check_market_health()
```

**Output**:
```
=== NSE Market Monitor (2026-09-19 14:30) ===

Market Status:
  Capital Market: Open
  Derivative Market: Open
  Currency Market: Open

FII/DII Activity (Latest):
  FII Buy: ₹4,500Cr | FII Sell: ₹3,200Cr
  DII Buy: ₹2,100Cr | DII Sell: ₹2,800Cr
  FII Net: ₹1,300Cr (BUYING)

Market Breadth:
  Advances: 1,850
  Declines: 1,200
  Unchanged: 350
  A/D Ratio: 1.54

Top Gainers:
  BANKEX: +2.45%
  NIFTY50: +1.23%
  SENSEX: +0.89%

Top Losers:
  PHARMA: -1.20%
  MIDCAP: -0.95%
  SMALLCAP: -0.78%
```

### 4. Simple Backtest (`backtest_simple_ma.py`)

Backtest a 10/20-day moving average crossover strategy.

**Features**:
- Fetch 1 year of historical data
- Apply MA crossover signals
- Calculate returns and trade statistics
- Print trade log

**Usage**:
```python
engine = BacktestEngine("RELIANCE")
results = engine.simple_ma_strategy(short_ma=10, long_ma=20)
```

**Output**:
```
=== Backtest: Simple MA Crossover ===

[2026-01-15] BUY 35 @ ₹2,800.00
[2026-02-20] SELL 35 @ ₹3,100.00
[2026-03-10] BUY 32 @ ₹3,050.00
...

=== Backtest Results ===
Symbol: RELIANCE
Initial Capital: ₹100,000
Final Value: ₹118,500
Returns: 18.50%
Total Trades: 8
  Buy: 4, Sell: 4
```

## Tips & Best Practices

### 1. Error Handling

Always wrap API calls in try-except:

```python
try:
    quote = client.get_equity_quote("RELIANCE")
except Exception as e:
    print(f"Error: {e}")
    # Handle gracefully
```

### 2. Rate Limiting

NSE can block high-frequency requests. Add delays:

```python
import time
time.sleep(0.5)  # 500ms between requests
```

Or use concurrent requests:

```python
import concurrent.futures

def fetch_quote(symbol):
    return client.get_equity_quote(symbol)

with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
    quotes = list(executor.map(fetch_quote, symbols))
```

### 3. Session Initialization

Initialize once at startup:

```python
client = financeindia.FinanceClient()
client._initialize_session()  # Warm up cookies
```

### 4. Logging

Add logging for production systems:

```python
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

logger.info(f"Fetching quote for {symbol}")
```

## Customization

Modify examples for your needs:

- **Portfolio Dashboard**: Replace symbols/quantities with your actual portfolio
- **Option Analysis**: Change `min_iv` threshold for different strategies
- **Market Monitor**: Add email/SMS alerts for conditions
- **Backtest**: Implement different entry/exit logic

## Performance Tips

1. **Reuse Client**: Create once, use across multiple requests
2. **Batch Operations**: Fetch multiple symbols in concurrent threads
3. **Cache Data**: Save quotes/data locally to avoid repeated calls
4. **Async Optional**: Currently sync API; async support coming in future versions

## Troubleshooting

### Import Error: `ModuleNotFoundError: No module named 'financeindia'`
```bash
pip install financeindia
```

### Network Error: `ConnectionError` or `Timeout`
- Check NSE website is accessible: https://www.nseindia.com/
- NSE may have blocked your IP (rate limiting)
- Try again after 1-2 hours
- Add backoff/retry logic

### Data Error: `ValueError: no data`
- Ensure date range is valid (trading days only)
- Symbol may not exist or may be delisted
- Try a different date range

## More Resources

- [README](../README.md) — Main documentation
- [EXAMPLES.md](../EXAMPLES.md) — More detailed code examples
- [BENCHMARKS.md](../BENCHMARKS.md) — Performance comparisons
- [ARCHITECTURE.md](../ARCHITECTURE.md) — Technical deep-dive

## Contributing

Found a bug or want to improve examples? Create an issue or PR on [GitHub](https://github.com/sandeep-jaiswar/financeindia).

---

**Happy trading! 📈**
