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
                print(f"  {mkt.market}: {mkt.status}")
        except Exception as e:
            print(f"Market Status Error: {e}")

        try:
            # 2. FII/DII Activity
            fii_dii = self.client.get_fii_dii_activity()
            print(f"\nFII/DII Activity (Latest):")
            if fii_dii:
                # Data comes as list of categories (FII, DII, etc.)
                for row in fii_dii[:2]:  # Show FII and DII
                    print(f"  {row.category}:")
                    print(f"    Buy: ₹{row.buy_value:,.0f}Cr | Sell: ₹{row.sell_value:,.0f}Cr")
                    print(f"    Net: ₹{row.net_value:,.0f}Cr {'(BUYING)' if row.net_value > 0 else '(SELLING)'}")
        except Exception as e:
            print(f"FII/DII Error: {e}")

        try:
            # 3. Market Breadth
            advances_declines = self.client.get_advances_declines()
            if advances_declines:
                advance = advances_declines.get('advance', {})
                print(f"\nMarket Breadth:")
                print(f"  Advances: {advance.get('advances', 0)}")
                print(f"  Declines: {advance.get('declines', 0)}")
                print(f"  Unchanged: {advance.get('unchanged', 0)}")
                if advance.get('declines', 0) > 0:
                    breadth_ratio = advance.get('advances', 0) / advance.get('declines', 1)
                    print(f"  A/D Ratio: {breadth_ratio:.2f}")
            else:
                print("  (No breadth data available)")
        except Exception as e:
            print(f"Breadth Error: {e}")

        try:
            # 4. Top Gainers/Losers
            gainers = self.client.get_top_gainers()["NIFTY"]["data"]
            losers = self.client.get_top_losers()["NIFTY"]["data"]

            print(f"\nTop Gainers:")
            for stock in gainers[:3] if gainers else []:
                print(f"  {stock.get('symbol', 'N/A')}: +{stock.get('pctChange', 0):.2f}%")

            print(f"\nTop Losers:")
            for stock in losers[:3] if losers else []:
                print(f"  {stock.get('symbol', 'N/A')}: {stock.get('pctChange', 0):.2f}%")
        except Exception as e:
            print(f"Gainers/Losers Error: {e}")


if __name__ == "__main__":
    monitor = MarketMonitor()
    monitor.check_market_health()
