import financeindia
import pytest

def test_market_stream_ssrf():
    # Valid domains
    financeindia.MarketStream("wss://nseindia.com/stream")
    financeindia.MarketStream("ws://www.mcxindia.com/stream")

    # Invalid scheme
    with pytest.raises(ValueError):
        financeindia.MarketStream("https://nseindia.com/stream")

    # Invalid domain
    with pytest.raises(ValueError):
        financeindia.MarketStream("wss://evil.com/stream")

test_market_stream_ssrf()
print("Tests ran")
