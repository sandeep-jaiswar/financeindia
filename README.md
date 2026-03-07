# financeindia

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)](https://www.python.org/)

**financeindia** is a high-performance, lightweight Python library written in Rust for fetching Indian financial market data (NSE) with ease.

## Why financeindia?

- **Blazing Fast**: Powered by Rust, providing efficient data parsing and networking.
- **Lightweight**: Minimal dependencies, focus on performance and reliability.
- **Session Caching**: Intelligent session management to avoid redundant requests and minimize rate-limiting risk.
- **Comprehensive**: Access to equity lists, historical data, bhavcopies, indices, and real-time quotes.

## Installation

```bash
pip install financeindia
```

## Quickstart

```python
import financeindia
import json

# Initialize the client
client = financeindia.FinanceClient()

# Recommended: initialize the session once
client._initialize_session()

# Get market status
status = client.get_market_status()
print(json.loads(status))

# Fetch historical data for a stock
data = client.price_volume_data("RELIANCE", "01-03-2025", "05-03-2025")
print(data)

# Fetch current quote
quote = client.get_equity_quote("RELIANCE")
print(json.loads(quote))
```

## Features

- **Capital Markets**: Equity lists (All & Nifty 50), historical price/volume, deliverable positions.
- **Bulk & Block Deals**: Track large institutional trades.
- **Bhavcopy**: Full daily trading data in UDiFF format.
- **Indices**: Comprehensive list of all NSE indices and their constituents.
- **Live Analysis**: Top gainers, top losers, and most active securities.
- **Derivatives**: Real-time option chain data for symbols and indices.
- **Market Info**: Holidays, corporate actions, and overall market status.

## Performance Highlights

`financeindia` implements a thread-safe session caching mechanism. The connection to NSE is refreshed only once every 15 minutes or when the session expires, ensuring that multiple API calls are as fast as possible.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.