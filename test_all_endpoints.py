import financeindia
import json
from datetime import datetime

def test_all():
    client = financeindia.FinanceClient()
    client._initialize_session()
    
    results = {
        "Macro": [],
        "Equities": [],
        "Indices": [],
        "Derivatives": [],
        "Corporate": [],
        "SLB": [],
        "Surveillance": []
    }

    def run_test(category, name, func, *args):
        print(f"Testing {name}...", end=" ", flush=True)
        try:
            data = func(*args)
            print("✅ SUCCESS")
            # data is now a list or dict
            info = len(data) if isinstance(data, (list, dict)) else "N/A"
            results[category].append({"name": name, "status": "PASS", "info": info})
        except Exception as e:
            print(f"❌ FAILED: {str(e)[:50]}...")
            results[category].append({"name": name, "status": "FAIL", "error": str(e)})

    # Dates
    test_date = "09-03-2026"
    from_date = "02-03-2026"
    to_date = "06-03-2026"

    # --- Macro ---
    run_test("Macro", "get_market_status", client.get_market_status)
    run_test("Macro", "get_holidays", client.get_holidays)
    run_test("Macro", "get_fii_dii_activity", client.get_fii_dii_activity)
    run_test("Macro", "get_market_turnover", client.get_market_turnover)
    run_test("Macro", "get_fii_stats", client.get_fii_stats, test_date)

    # --- Equities ---
    run_test("Equities", "get_equity_list", client.get_equity_list)
    run_test("Equities", "price_volume_data", client.price_volume_data, "RELIANCE", from_date, to_date)
    run_test("Equities", "deliverable_position_data", client.deliverable_position_data, "RELIANCE", from_date, to_date)
    run_test("Equities", "bhav_copy_equities", client.bhav_copy_equities, test_date)
    run_test("Equities", "bulk_deal_data", client.bulk_deal_data, from_date, to_date)
    run_test("Equities", "block_deals_data", client.block_deals_data, from_date, to_date)
    run_test("Equities", "short_selling_data", client.short_selling_data, from_date, to_date)
    run_test("Equities", "get_52week_high_low", client.get_52week_high_low, "high")
    run_test("Equities", "get_top_gainers", client.get_top_gainers)
    run_test("Equities", "get_top_losers", client.get_top_losers)
    run_test("Equities", "get_most_active", client.get_most_active, "NIFTY 50")
    run_test("Equities", "get_advances_declines", client.get_advances_declines)
    run_test("Equities", "get_monthly_settlement_stats", client.get_monthly_settlement_stats, "2025-2026")
    run_test("Equities", "get_equity_quote", client.get_equity_quote, "RELIANCE")

    # --- Indices ---
    run_test("Indices", "get_all_indices", client.get_all_indices)
    run_test("Indices", "get_index_constituents", client.get_index_constituents, "NIFTY 50")
    run_test("Indices", "get_index_history", client.get_index_history, "NIFTY 50", from_date, to_date)
    run_test("Indices", "get_index_yield", client.get_index_yield, "NIFTY 50", from_date, to_date)
    run_test("Indices", "get_india_vix_history", client.get_india_vix_history, from_date, to_date)
    run_test("Indices", "get_total_returns_index", client.get_total_returns_index, "NIFTY 50", from_date, to_date)

    # --- Derivatives ---
    run_test("Derivatives", "get_option_chain", client.get_option_chain, "NIFTY", True)
    run_test("Derivatives", "bhav_copy_derivatives_fo", client.bhav_copy_derivatives, test_date, "FO")
    run_test("Derivatives", "get_fo_sec_ban", client.get_fo_sec_ban)
    run_test("Derivatives", "get_span_margins", client.get_span_margins, test_date)

    # --- Corporate ---
    run_test("Corporate", "get_corporate_actions", client.get_corporate_actions)
    run_test("Corporate", "get_financial_results", client.get_financial_results, "RELIANCE", "01-01-2025", "31-12-2025", "Quarterly")

    # --- SLB ---
    run_test("SLB", "get_slb_bhavcopy", client.get_slb_bhavcopy, test_date)
    run_test("SLB", "get_slb_eligible", client.get_slb_eligible)
    run_test("SLB", "get_slb_series_master", client.get_slb_series_master)
    run_test("SLB", "get_slb_open_positions", client.get_slb_open_positions, "MAR2026")

    # --- Surveillance ---
    run_test("Surveillance", "get_asm_stocks", client.get_asm_stocks)
    run_test("Surveillance", "get_gsm_stocks", client.get_gsm_stocks)
    run_test("Surveillance", "get_short_ban_stocks", client.get_short_ban_stocks)

    # --- Additional / Granular ---
    run_test("Derivatives", "get_fo_ban_list", client.get_fo_ban_list, test_date)
    run_test("Derivatives", "get_participant_volume", client.get_participant_volume, test_date)
    run_test("Derivatives", "get_oi_limits_cli", client.get_oi_limits_cli, test_date)
    run_test("Corporate", "get_insider_trades", client.get_insider_trades, from_date, to_date)

    # Summary
    print("\n" + "="*50)
    print("TEST SUMMARY")
    print("="*50)
    total_pass = 0
    total_fail = 0
    for cat, tests in results.items():
        pass_count = sum(1 for t in tests if t["status"] == "PASS")
        fail_count = len(tests) - pass_count
        total_pass += pass_count
        total_fail += fail_count
        print(f"{cat:12}: {pass_count} PASS, {fail_count} FAIL")
    
    print("-" * 50)
    print(f"TOTAL       : {total_pass} PASS, {total_fail} FAIL")
    print("="*50)

    with open("test_results.json", "w") as f:
        json.dump(results, f, indent=2)

if __name__ == "__main__":
    test_all()
