#!/usr/bin/env python3
"""
Simple Backtest: Moving Average Crossover

Backtest a simple 10/20-day MA crossover strategy.
"""

import financeindia
from datetime import datetime, timedelta


class BacktestEngine:
    def __init__(self, symbol: str, initial_capital: float = 100000):
        self.client = financeindia.FinanceClient()
        self.symbol = symbol
        self.capital = initial_capital
        self.cash = initial_capital
        self.positions = 0
        self.trades = []

    def fetch_historical_data(self, days: int = 252) -> list:
        """Fetch last N trading days"""
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)

        try:
            data = self.client.price_volume_data(
                self.symbol,
                start_date.strftime("%d-%m-%Y"),
                end_date.strftime("%d-%m-%Y")
            )
            return sorted(data, key=lambda x: x.date)
        except Exception as e:
            print(f"Error fetching data: {e}")
            return []

    def simple_ma_strategy(self, short_ma: int = 10, long_ma: int = 20):
        """Simple moving average crossover strategy"""
        data = self.fetch_historical_data(252)

        if len(data) < long_ma:
            print(f"Not enough data (need {long_ma}, got {len(data)})")
            return {}

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
            })
            print(f"[{date}] BUY {shares} @ ₹{price:.2f}")

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
            })
            print(f"[{date}] SELL {self.positions} @ ₹{price:.2f}")
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
    print(f"\n=== Backtest: Simple MA Crossover ===")
    engine = BacktestEngine("RELIANCE")
    results = engine.simple_ma_strategy(short_ma=10, long_ma=20)

    if results:
        print(f"\n=== Backtest Results ===")
        print(f"Symbol: {results['symbol']}")
        print(f"Initial Capital: ₹{results['initial_capital']:,.0f}")
        print(f"Final Value: ₹{results['final_value']:,.2f}")
        print(f"Returns: {results['returns_pct']:.2f}%")
        print(f"Total Trades: {results['total_trades']}")
        print(f"  Buy: {results['buy_trades']}, Sell: {results['sell_trades']}")
