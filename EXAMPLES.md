# Examples: Real-World Usage

This guide provides production-ready code examples for common use cases.

## Table of Contents

1. [Real-time Portfolio Dashboard](#real-time-portfolio-dashboard)
2. [Algorithmic Trading Bot](#algorithmic-trading-bot)
3. [Option Chain Analysis](#option-chain-analysis)
4. [Financial Analysis with XBRL](#financial-analysis-with-xbrl)
5. [Market Monitoring System](#market-monitoring-system)
6. [Backtesting Framework](#backtesting-framework)

---

## Real-time Portfolio Dashboard

Update a portfolio with current market prices and calculate P&L.

```python
import financeindia
from dataclasses import dataclass
from typing import List

@dataclass
class Position:
    symbol: str
    quantity: int
    buy_price: float

class PortfolioDashboard:
    def __init__(self):
        self.client = financeindia.FinanceClient()
        self.positions: List[Position] = [
            Position("RELIANCE", 100, 2850),
            Position("INFY", 50, 3400),
            Position("TCS", 25, 3800),
        ]
    
    def update_prices(self):
        """Fetch current quotes for all positions"""
        total_value = 0
        total_pl = 0
        
        print(f"\n{'Symbol':<10} {'Qty':<6} {'Buy':<10} {'Current':<10} {'P&L':<12} {'P&L %':<8}")
        print("-" * 66)
        
        for position in self.positions:
            quote = self.client.get_equity_quote(position.symbol)
            current_price = quote['tradeInfo']['lastPrice']
            
            position_value = position.quantity * current_price
            buy_value = position.quantity * position.buy_price
            pl = position_value - buy_value
            pl_pct = (pl / buy_value) * 100
            
            total_value += position_value
            total_pl += pl
            
            print(f"{position.symbol:<10} {position.quantity:<6} {position.buy_price:<10.2f} "
                  f"{current_price:<10.2f} {pl:<12.2f} {pl_pct:<8.2f}%")
        
        total_pl_pct = (total_pl / sum(p.quantity * p.buy_price for p in self.positions)) * 100
        print("-" * 66)
        print(f"{'TOTAL':<10} {'':6} {'':10} {'':10} {total_pl:<12.2f} {total_pl_pct:<8.2f}%")
        print(f"Portfolio Value: ₹{total_value:,.2f}")

if __name__ == "__main__":
    dashboard = PortfolioDashboard()
    dashboard.update_prices()
```

---

## Algorithmic Trading Bot

Simple mean-reversion bot using 20-day moving average.

```python
import financeindia
from statistics import mean
from datetime import datetime, timedelta

class MeanReversionBot:
    def __init__(self, symbol: str, lookback_days: int = 20):
        self.client = financeindia.FinanceClient()
        self.symbol = symbol
        self.lookback_days = lookback_days
    
    def calculate_signal(self) -> str:
        """
        Returns: 'BUY', 'SELL', or 'HOLD'
        """
        # Fetch last 20 days
        end_date = datetime.now()
        start_date = end_date - timedelta(days=self.lookback_days)
        
        data = self.client.price_volume_data(
            self.symbol,
            start_date.strftime("%d-%m-%Y"),
            end_date.strftime("%d-%m-%Y")
        )
        
        if len(data) < self.lookback_days:
            return "HOLD"
        
        # Calculate 20-day MA
        closes = [row.close_price for row in data]
        ma_20 = mean(closes)
        current_price = closes[-1]
        
        # Get current quote for volume confirmation
        quote = self.client.get_equity_quote(self.symbol)
        volume = quote['tradeInfo']['totalTradedVolume']
        avg_volume = mean([row.quantity for row in data]) if data else 0
        
        # Signal logic
        if current_price < ma_20 * 0.95 and volume > avg_volume * 1.2:
            return "BUY"  # Price 5% below MA + high volume
        elif current_price > ma_20 * 1.05 and volume > avg_volume * 1.2:
            return "SELL"  # Price 5% above MA + high volume
        else:
            return "HOLD"
    
    def run(self):
        signal = self.calculate_signal()
        quote = self.client.get_equity_quote(self.symbol)
        price = quote['tradeInfo']['lastPrice']
        
        print(f"{self.symbol} @ ₹{price:.2f} → {signal}")
        return signal

if __name__ == "__main__":
    bot = MeanReversionBot("RELIANCE")
    signal = bot.run()
```

---

## Option Chain Analysis

Analyze option chains to find high-IV opportunities.

```python
import financeindia
from dataclasses import dataclass
from typing import Optional

@dataclass
class OptionOpportunity:
    strike: float
    type: str  # 'CE' or 'PE'
    iv: float
    bid: float
    ask: float
    delta: Optional[float]
    theta: Optional[float]

class OptionChainAnalyzer:
    def __init__(self, index: str = "NIFTY"):
        self.client = financeindia.FinanceClient()
        self.index = index
    
    def find_high_iv_opportunities(self, min_iv: float = 30) -> list:
        """
        Find option strikes with IV > min_iv
        """
        options = self.client.get_option_chain(self.index, is_index=True)
        
        opportunities = []
        for row in options:
            # High IV = higher option premiums
            if hasattr(row, 'iv') and row.iv > min_iv:
                opp = OptionOpportunity(
                    strike=row.strike,
                    type="CE" if row.option_type == "CE" else "PE",
                    iv=row.iv,
                    bid=row.bid,
                    ask=row.ask,
                    delta=row.delta if hasattr(row, 'delta') else None,
                    theta=row.theta if hasattr(row, 'theta') else None,
                )
                opportunities.append(opp)
        
        return sorted(opportunities, key=lambda x: x.iv, reverse=True)[:5]
    
    def print_report(self):
        print(f"\n=== High IV Opportunities ({self.index}) ===\n")
        opportunities = self.find_high_iv_opportunities()
        
        for opp in opportunities:
            delta = f"{opp.delta:>6.2f}" if opp.delta is not None else "   N/A"
            theta = f"{opp.theta:>6.2f}" if opp.theta is not None else "   N/A"
            print(f"Strike {opp.strike:>6.0f} {opp.type} | "
                  f"IV: {opp.iv:>5.1f}% | "
                  f"Bid: {opp.bid:>6.2f} | "
                  f"Ask: {opp.ask:>6.2f} | "
                  f"Delta: {delta} | "
                  f"Theta: {theta}")

if __name__ == "__main__":
    analyzer = OptionChainAnalyzer("NIFTY")
    analyzer.print_report()
```

---

## Financial Analysis with XBRL

Extract fundamental financial data for investment analysis.

```python
import financeindia
from datetime import datetime

class FundamentalAnalyzer:
    def __init__(self, symbol: str):
        self.client = financeindia.FinanceClient()
        self.symbol = symbol
    
    def analyze_financials(self):
        """
        Fetch and analyze latest annual financial results
        """
        # Get latest annual filing
        results = self.client.get_financial_results(
            self.symbol,
            "01-01-2024",
            datetime.now().strftime("%d-%m-%Y"),
            "Annual"  # or "Quarterly"
        )
        
        if not results:
            print(f"No financial results found for {self.symbol}")
            return
        
        latest = results[0]
        print(f"\n=== {self.symbol} Financial Analysis ===")
        print(f"Filing Date: {latest.get('filingDate')}")
        print(f"Period: {latest.get('period')}")
        print(f"XBRL URL: {latest.get('xbrl')}")
        
        # Parse detailed XBRL data
        xbrl_url = latest['xbrl']
        financial_data = self.client.get_financial_details(xbrl_url)
        
        # Extract key metrics (structure varies; adjust as needed)
        if isinstance(financial_data, dict):
            # Print top-level categories
            for key, value in list(financial_data.items())[:10]:
                print(f"{key}: {value}")
    
    def get_key_ratios(self) -> dict:
        """
        Calculate basic financial ratios from XBRL
        (requires parsing the detailed response)
        """
        results = self.client.get_financial_results(
            self.symbol, "01-01-2024", datetime.now().strftime("%d-%m-%Y"), "Annual"
        )
        if not results:
            return {}
        
        xbrl_url = results[0]['xbrl']
        data = self.client.get_financial_details(xbrl_url)
        
        # Example: calculate ratios from data
        # Structure depends on XBRL response
        ratios = {
            "symbol": self.symbol,
            "source": "XBRL",
            "note": "Parse 'data' dict for revenue, net_profit, assets, etc."
        }
        return ratios

if __name__ == "__main__":
    analyzer = FundamentalAnalyzer("TCS")
    analyzer.analyze_financials()
    ratios = analyzer.get_key_ratios()
    print(f"\nKey Ratios: {ratios}")
```

---

## Market Monitoring System

Monitor FII/DII activity and market breadth.

```python
import financeindia
from datetime import datetime

class MarketMonitor:
    def __init__(self):
        self.client = financeindia.FinanceClient()
    
    def check_market_health(self):
        """
        High-level market health check
        """
        print(f"\n=== NSE Market Monitor ({datetime.now().strftime('%Y-%m-%d %H:%M')}) ===\n")
        
        # 1. Market Status
        status = self.client.get_market_status()
        print(f"Market Status:")
        for mkt in status.market_state[:3]:  # First 3 segments
            print(f"  {mkt.name}: {mkt.status}")
        
        # 2. FII/DII Activity
        fii_dii = self.client.get_fii_dii_activity()
        print(f"\nFII/DII Activity (Latest):")
        for row in fii_dii[:1]:  # Most recent
            print(f"  FII Buy: ₹{row.fii_buy_value:,.0f}Cr | FII Sell: ₹{row.fii_sell_value:,.0f}Cr")
            print(f"  DII Buy: ₹{row.dii_buy_value:,.0f}Cr | DII Sell: ₹{row.dii_sell_value:,.0f}Cr")
            print(f"  FII Net: ₹{row.fii_buy_value - row.fii_sell_value:,.0f}Cr")
        
        # 3. Market Breadth
        breadth = self.client.get_advances_declines()
        print(f"\nMarket Breadth:")
        print(f"  Advances: {breadth.advances}")
        print(f"  Declines: {breadth.declines}")
        print(f"  Unchanged: {breadth.unchanged}")
        breadth_ratio = breadth.advances / max(breadth.declines, 1)
        print(f"  A/D Ratio: {breadth_ratio:.2f}")
        
        # 4. Top Gainers/Losers
        gainers = self.client.get_top_gainers()
        losers = self.client.get_top_losers()
        print(f"\nTop Gainers:")
        for stock in gainers[:3]:
            print(f"  {stock.symbol}: +{stock.pct_change:.2f}%")
        print(f"\nTop Losers:")
        for stock in losers[:3]:
            print(f"  {stock.symbol}: {stock.pct_change:.2f}%")

if __name__ == "__main__":
    monitor = MarketMonitor()
    monitor.check_market_health()
```

---

## Backtesting Framework

Simple backtesting engine for strategy validation.

```python
import financeindia
from datetime import datetime, timedelta
from typing import List, Tuple

class BacktestEngine:
    def __init__(self, symbol: str, initial_capital: float = 100000):
        self.client = financeindia.FinanceClient()
        self.symbol = symbol
        self.capital = initial_capital
        self.cash = initial_capital
        self.positions = 0
        self.trades: List[dict] = []
    
    def fetch_historical_data(self, days: int = 252) -> list:
        """Fetch last N trading days"""
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        
        data = self.client.price_volume_data(
            self.symbol,
            start_date.strftime("%d-%m-%Y"),
            end_date.strftime("%d-%m-%Y")
        )
        return sorted(data, key=lambda x: x.date)
    
    def simple_ma_strategy(self, short_ma: int = 10, long_ma: int = 20):
        """
        Simple moving average crossover strategy
        """
        data = self.fetch_historical_data(252)
        
        for i in range(long_ma, len(data)):
            previous_window = data[i-long_ma:i]
            current_window = data[i-long_ma+1:i+1]
            previous_prices = [row.close_price for row in previous_window]
            current_prices = [row.close_price for row in current_window]
            
            previous_short_avg = sum(previous_prices[-short_ma:]) / short_ma
            previous_long_avg = sum(previous_prices) / long_ma
            current_short_avg = sum(current_prices[-short_ma:]) / short_ma
            current_long_avg = sum(current_prices) / long_ma
            
            current_price = data[i].close_price
            
            # BUY signal: short MA crosses above long MA
            if (previous_short_avg <= previous_long_avg and
                    current_short_avg > current_long_avg and self.positions == 0):
                self.buy(current_price, data[i].date)
            
            # SELL signal: short MA crosses below long MA
            elif (previous_short_avg >= previous_long_avg and
                  current_short_avg < current_long_avg and self.positions > 0):
                self.sell(current_price, data[i].date)
        
        final_close = data[-1].close_price
        return self.calculate_returns(final_close)
    
    def buy(self, price: float, date: str):
        """Buy signal"""
        shares = int(self.cash / price)
        if shares > 0:
            self.positions = shares
            self.cash -= shares * price
            self.trades.append({
                'date': date,
                'type': 'BUY',
                'price': price,
                'shares': shares,
                'total': shares * price
            })
    
    def sell(self, price: float, date: str):
        """Sell signal"""
        if self.positions > 0:
            proceeds = self.positions * price
            self.cash += proceeds
            self.trades.append({
                'date': date,
                'type': 'SELL',
                'price': price,
                'shares': self.positions,
                'total': proceeds
            })
            self.positions = 0
    
    def calculate_returns(self, final_close: float) -> dict:
        """Calculate backtest performance metrics"""
        final_value = self.cash + (self.positions * final_close if self.positions > 0 else 0)
        returns = ((final_value - self.capital) / self.capital) * 100
        
        return {
            'symbol': self.symbol,
            'initial_capital': self.capital,
            'final_value': final_value,
            'returns_pct': returns,
            'total_trades': len(self.trades),
            'buy_trades': sum(1 for t in self.trades if t['type'] == 'BUY'),
            'sell_trades': sum(1 for t in self.trades if t['type'] == 'SELL'),
        }

if __name__ == "__main__":
    engine = BacktestEngine("RELIANCE")
    results = engine.simple_ma_strategy(short_ma=10, long_ma=20)
    
    print(f"\n=== Backtest Results ===")
    print(f"Symbol: {results['symbol']}")
    print(f"Initial Capital: ₹{results['initial_capital']:,.0f}")
    print(f"Final Value: ₹{results['final_value']:,.2f}")
    print(f"Returns: {results['returns_pct']:.2f}%")
    print(f"Total Trades: {results['total_trades']}")
```

---

## Tips & Best Practices

1. **Reuse Client**: Create client once, reuse across requests
2. **Handle Rate Limits**: NSE/Akamai may rate-limit; add backoff delays
3. **Concurrent Requests**: Use `ThreadPoolExecutor` for multiple symbols
4. **Session Initialization**: Call `client._initialize_session()` once at startup
5. **Error Handling**: Wrap API calls in try-except for network resilience
6. **Data Validation**: Always check if responses are None or empty

---

## Next Steps

- 📖 Read [ARCHITECTURE.md](ARCHITECTURE.md) for implementation details
- 🔍 Check [BENCHMARKS.md](BENCHMARKS.md) for performance metrics
- 📊 See [COMPARISON.md](COMPARISON.md) for alternative libraries
- 🤝 Contribute examples to [GitHub](https://github.com/sandeep-jaiswar/financeindia)

---

**Last Updated**: 2026-09-19  
**financeindia Version**: 0.2.3
