#!/usr/bin/env python3
"""
Real-time Portfolio Dashboard

Shows portfolio P&L with current market prices.
"""

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
        # Example positions (replace with your actual portfolio)
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
            try:
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
            except Exception as e:
                print(f"{position.symbol:<10} Error: {str(e)[:30]}")

        total_buy_value = sum(p.quantity * p.buy_price for p in self.positions)
        if total_buy_value > 0:
            total_pl_pct = (total_pl / total_buy_value) * 100
            print("-" * 66)
            print(f"{'TOTAL':<10} {'':6} {'':10} {'':10} {total_pl:<12.2f} {total_pl_pct:<8.2f}%")
            print(f"Portfolio Value: ₹{total_value:,.2f}")


if __name__ == "__main__":
    dashboard = PortfolioDashboard()
    dashboard.update_prices()
