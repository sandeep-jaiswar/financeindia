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

def test_equity_data_endpoints(client):
    # Test a few core equity data endpoints
    assert client.get_equity_quote("RELIANCE") is not None
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
    # An invalid ticker should result in a JSON parsing error (ValueError)
    # or a ConnectionError if the API returns a non-200 status.
    with pytest.raises(ValueError):
        client.get_equity_quote("INVALID_TICKER_9999")

def test_market_stream_ssrf_protection():
    # Only wss/ws and valid domains should be accepted

    # Invalid schemes
    with pytest.raises(RuntimeError, match="Only ws and wss URLs are allowed"):
        financeindia.MarketStream("https://nseindia.com/stream")
    with pytest.raises(RuntimeError, match="Only ws and wss URLs are allowed"):
        financeindia.MarketStream("http://nseindia.com/stream")

    # Invalid hosts
    with pytest.raises(RuntimeError, match="URL host must be a trusted domain"):
        financeindia.MarketStream("wss://evil.com/stream")
    with pytest.raises(RuntimeError, match="URL host must be a trusted domain"):
        financeindia.MarketStream("wss://nseindia.com.evil.com/stream")

    # Valid hosts
    # Creating an instance shouldn't raise exception during instantiation
    stream = financeindia.MarketStream("wss://nseindia.com/stream")
    assert stream is not None

    stream2 = financeindia.MarketStream("wss://mcxindia.com/stream")
    assert stream2 is not None
    # Valid URLs
    financeindia.MarketStream("wss://stream.nseindia.com/market")
    financeindia.MarketStream("ws://mcxindia.com/stream")

    # Invalid scheme
    with pytest.raises(ValueError, match="Only ws and wss URLs are allowed"):
        financeindia.MarketStream("https://stream.nseindia.com/market")

    # Invalid host
    with pytest.raises(ValueError, match="URL host must be a trusted NSE or MCX domain"):
        financeindia.MarketStream("wss://evil.com/stream")
