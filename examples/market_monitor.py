#!/usr/bin/env python3
"""
Market Monitoring System

Monitor FII/DII activity, breadth, and market status.
"""

import financeindia
from datetime import datetime


class MarketMonitor:
    def __init__(self):
        self.client = financeindia.FinanceClient()

    def check_market_health(self):
        """High-level market health check"""
        print(f"\n=== NSE Market Monitor ({datetime.now().strftime('%Y-%m-%d %H:%M')}) ===\n")

        try:
            # 1. Market Status
            status = self.client.get_market_status()
            print("Market Status:")
            for mkt in status.market_state[:3]:
                print(f"  {mkt.name}: {mkt.status}")
        except Exception as e:
            print(f"Market Status Error: {e}")

        try:
            # 2. FII/DII Activity
            fii_dii = self.client.get_fii_dii_activity()
            print(f"\nFII/DII Activity (Latest):")
            if fii_dii:
                row = fii_dii[0]
                print(f"  FII Buy: ₹{row.fii_buy_value:,.0f}Cr | FII Sell: ₹{row.fii_sell_value:,.0f}Cr")
                print(f"  DII Buy: ₹{row.dii_buy_value:,.0f}Cr | DII Sell: ₹{row.dii_sell_value:,.0f}Cr")
                fii_net = row.fii_buy_value - row.fii_sell_value
                print(f"  FII Net: ₹{fii_net:,.0f}Cr {'(BUYING)' if fii_net > 0 else '(SELLING)'}")
        except Exception as e:
            print(f"FII/DII Error: {e}")

        try:
            # 3. Market Breadth
            breadth = self.client.get_advances_declines()
            print(f"\nMarket Breadth:")
            print(f"  Advances: {breadth.advances}")
            print(f"  Declines: {breadth.declines}")
            print(f"  Unchanged: {breadth.unchanged}")
            if breadth.declines > 0:
                breadth_ratio = breadth.advances / breadth.declines
                print(f"  A/D Ratio: {breadth_ratio:.2f}")
        except Exception as e:
            print(f"Breadth Error: {e}")

        try:
            # 4. Top Gainers/Losers
            gainers = self.client.get_top_gainers()
            losers = self.client.get_top_losers()

            print(f"\nTop Gainers:")
            for stock in gainers[:3]:
                print(f"  {stock.symbol}: +{stock.pct_change:.2f}%")

            print(f"\nTop Losers:")
            for stock in losers[:3]:
                print(f"  {stock.symbol}: {stock.pct_change:.2f}%")
        except Exception as e:
            print(f"Gainers/Losers Error: {e}")


if __name__ == "__main__":
    monitor = MarketMonitor()
    monitor.check_market_health()
