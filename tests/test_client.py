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
    assert hasattr(data[0], 'symbol')

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

# SLB Module
def test_slb_eligible(client):
    assert client.get_slb_eligible() is not None

# Error Handling Exception Tests
def test_missing_data_exception(client):
    with pytest.raises(Exception):
        # A bad symbol quote should raise Python RuntimeError from PyErr
        client.get_equity_quote("INVALID_TICKER_9999")
