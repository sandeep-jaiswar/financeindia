import financeindia
import time

def test_scrapers():
    print("Initializing session...")
    client = financeindia.FinanceClient()
    client._initialize_session()
    
    # 1. Testing get_market_status
    print("\n1. Testing get_market_status...")
    try:
        status = client.get_market_status()
        print(f"Market Status: {status[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    # 2. Testing get_index_history (NIFTY 50)
    print("\n2. Testing get_index_history (NIFTY 50)...")
    try:
        history = client.get_index_history("NIFTY 50", "02-03-2026", "05-03-2026")
        print(f"Index History: {history[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    # 3. Testing get_index_yield (NIFTY 50)
    print("\n3. Testing get_index_yield (NIFTY 50)...")
    try:
        yield_data = client.get_index_yield("NIFTY 50", "02-03-2026", "05-03-2026")
        print(f"Index Yield: {yield_data[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    # 4. Testing get_india_vix_history
    print("\n4. Testing get_india_vix_history...")
    try:
        vix = client.get_india_vix_history("02-03-2026", "05-03-2026")
        print(f"India VIX: {vix[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    # 5. Testing get_fii_dii_activity
    print("\n5. Testing get_fii_dii_activity...")
    try:
        activity = client.get_fii_dii_activity()
        print(f"FII/DII Activity: {activity[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    # 6. Testing get_52week_high_low (high)
    print("\n6. Testing get_52week_high_low (high)...")
    try:
        data = client.get_52week_high_low("high")
        print(f"52W High/Low: {data[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    # 7. Testing SLB Bhavcopy
    print("\n7. Testing get_slb_bhavcopy...")
    try:
        data = client.get_slb_bhavcopy("09-03-2026")
        print(f"SLB Bhavcopy length: {len(data)}")
    except Exception as e:
        print(f"Failed: {e}")

    # 8. Testing Derivatives Bhavcopy (FO)
    print("\n8. Testing bhav_copy_derivatives (FO)...")
    try:
        data = client.bhav_copy_derivatives("09-03-2026", "FO")
        print(f"FO Bhavcopy length: {len(data)}")
    except Exception as e:
        print(f"Failed: {e}")

    # 9. Testing Total Returns Index
    print("\n9. Testing get_total_returns_index...")
    try:
        data = client.get_total_returns_index("NIFTY 50", "02-03-2026", "05-03-2026")
        print(f"TRI Data: {data[:100]}...")
    except Exception as e:
        print(f"Failed: {e}")

    print("\nAll verification steps completed.")

if __name__ == "__main__":
    test_scrapers()
