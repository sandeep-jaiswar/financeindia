import pytest
import financeindia

@pytest.fixture(scope="module")
def client():
    # Only init client once for the whole integration suite 
    # to avoid excessive connection setups
    return financeindia.FinanceClient()

# Core Macro & Utilities
def test_market_status(client):
    res = client.get_market_status()
    assert len(res.market_state) > 0

def test_holidays(client):
    assert client.get_holidays() is not None

# Equities Module
@pytest.mark.parametrize("mode", ["high", "low"])
def test_52_week(client, mode):
    data = client.get_52week_high_low(mode)
    assert data is not None

def test_top_gainers_losers(client):
    assert client.get_top_gainers() is not None
    assert client.get_top_losers() is not None

def test_equity_list(client):
    data = client.get_equity_list()
    assert len(data) > 0
    # Regression: company_name/isin used to deserialize to None because serde
    # renames never matched the real EQUITY_L.csv headers.
    assert all(row.symbol is not None for row in data)
    assert any(row.company_name is not None for row in data)
    assert any(row.isin is not None for row in data)

def test_price_volume_data(client):
    data = client.price_volume_data("RELIANCE", "01-03-2026", "05-03-2026")
    assert len(data) > 0
    row = data[0]
    assert row.symbol == "RELIANCE"
    assert row.close_price is not None
    assert isinstance(row.close_price, float)
    # Quantity/count fields are integers, not floats.
    assert row.total_traded_quantity is None or isinstance(row.total_traded_quantity, int)
    assert row.no_of_trades is None or isinstance(row.no_of_trades, int)

def test_equity_data_endpoints(client):
    # Test a few core equity data endpoints.
    # Note: get_equity_quote is tested separately; it can be environment-dependent
    # and may fail with HTTP 403 from datacenter IPs due to NSE/Akamai bot detection.
    assert client.get_most_active("NIFTY 50") is not None
    assert client.get_advances_declines() is not None

# Indices Module
def test_indices_endpoints(client):
    assert client.get_all_indices() is not None
    assert client.get_index_constituents("NIFTY 50") is not None

# Derivatives Module
def test_fo_sec_ban(client):
    data = client.get_fo_sec_ban()
    assert data is not None

def test_live_currency_market(client):
    data = client.get_live_currency_market()
    assert data is not None
    assert "data" in data

def test_currency_bhavcopy(client):
    data = client.get_currency_bhavcopy("30-07-2026")
    assert isinstance(data, dict)
    assert "TradDt" in data

def test_live_commodities_market(client):
    data = client.get_live_commodities_market()
    assert data is not None
    assert "marketStatus" in data

def test_equity_quote(client):
    quote = client.get_equity_quote("RELIANCE")
    assert isinstance(quote, dict)
    assert "tradeInfo" in quote
    assert "priceInfo" in quote
    assert quote["tradeInfo"]["lastPrice"] is not None

def test_nse_commodities_bhavcopy(client):
    data = client.get_nse_commodities_bhavcopy("30-07-2026")
    assert isinstance(data, dict)
    assert "TradDt" in data

def test_mcx_bhavcopy(client):
    data = client.get_mcx_bhavcopy("30-07-2026")
    assert isinstance(data, list)
    assert len(data) > 0
    row = data[0]
    assert isinstance(row, dict)
    assert "Symbol" in row
    assert "Close" in row

@pytest.mark.parametrize("symbol,is_index", [
    ("RELIANCE", False),
    ("NIFTY", True)
])
def test_option_chain(client, symbol, is_index):
    data = client.get_option_chain(symbol, is_index)
    assert data is not None

# Corporate & Surveillance
def test_corporate_surveillance(client):
    assert client.get_corporate_actions() is not None
    assert client.get_gsm_stocks() is not None
    assert client.get_asm_stocks() is not None

# SLB Module
def test_slb_endpoints(client):
    assert client.get_slb_eligible() is not None
    assert client.get_slb_series_master() is not None

# Error Handling Exception Tests
def test_missing_data_exception(client):
    # Invalid option chain symbol should surface as a DataError, not crash.
    with pytest.raises(financeindia.FinanceError):
        client.get_option_chain("INVALID_TICKER_9999", True)

def test_market_stream_ssrf_protection(monkeypatch):
    # Only wss (and ws in the FINANCEINDIA_TEST_ENV=1 test mode) and valid
    # domains should be accepted.

    # Ensure test environment variable is not set to guarantee ws:// is rejected
    monkeypatch.delenv("FINANCEINDIA_TEST_ENV", raising=False)

    # Invalid schemes - code raises ValueError (not RuntimeError as originally expected)
    with pytest.raises(ValueError, match="Invalid URL scheme"):
        financeindia.MarketStream("https://nseindia.com/stream")
    with pytest.raises(ValueError, match="Invalid URL scheme"):
        financeindia.MarketStream("http://nseindia.com/stream")
    # Plain ws is rejected by default; only FINANCEINDIA_TEST_ENV=1 allows it.
    with pytest.raises(ValueError, match="Invalid URL scheme"):
        financeindia.MarketStream("ws://mcxindia.com/stream")

    # Invalid hosts - code raises ValueError
    with pytest.raises(ValueError, match="Invalid domain"):
        financeindia.MarketStream("wss://evil.com/stream")
    with pytest.raises(ValueError, match="Invalid domain"):
        financeindia.MarketStream("wss://nseindia.com.evil.com/stream")

    # Valid hosts
    # Creating an instance shouldn't raise exception during instantiation
    stream = financeindia.MarketStream("wss://nseindia.com/stream")
    assert stream is not None

    stream2 = financeindia.MarketStream("wss://mcxindia.com/stream")
    assert stream2 is not None
    # Valid URLs
    financeindia.MarketStream("wss://stream.nseindia.com/market")

    # Invalid scheme - already covered above
    # Invalid host - already covered above
