#!/usr/bin/env python3
"""
Option Chain Analysis

Analyze option chains to find high IV opportunities.
"""

import financeindia
from dataclasses import dataclass


@dataclass
class OptionOpportunity:
    strike: float
    option_type: str
    iv: float
    bid: float
    ask: float


class OptionChainAnalyzer:
    def __init__(self, index: str = "NIFTY"):
        self.client = financeindia.FinanceClient()
        self.index = index

    def find_high_iv_opportunities(self, min_iv: float = 30) -> list:
        """Find option strikes with IV > min_iv"""
        try:
            options = self.client.get_option_chain(self.index, is_index=True)

            opportunities = []
            for row in options:
                # High IV = higher option premiums = selling opportunities
                if hasattr(row, 'iv') and row.iv > min_iv:
                    opp = OptionOpportunity(
                        strike=row.strike,
                        option_type="CE" if row.option_type == "CE" else "PE",
                        iv=row.iv,
                        bid=row.bid if hasattr(row, 'bid') else 0,
                        ask=row.ask if hasattr(row, 'ask') else 0,
                    )
                    opportunities.append(opp)

            return sorted(opportunities, key=lambda x: x.iv, reverse=True)[:5]
        except Exception as e:
            print(f"Error fetching option chain: {e}")
            return []

    def print_report(self):
        print(f"\n=== High IV Opportunities ({self.index}) ===\n")
        opportunities = self.find_high_iv_opportunities()

        if not opportunities:
            print("No opportunities found or data unavailable")
            return

        print(f"{'Strike':<8} {'Type':<4} {'IV':<8} {'Bid':<8} {'Ask':<8}")
        print("-" * 40)
        for opp in opportunities:
            print(f"{opp.strike:>6.0f}   {opp.option_type:<4} {opp.iv:>6.1f}%  "
                  f"{opp.bid:>6.2f}  {opp.ask:>6.2f}")


if __name__ == "__main__":
    analyzer = OptionChainAnalyzer("NIFTY")
    analyzer.print_report()
