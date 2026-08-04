"""Python-side MCX bhavcopy fetcher.

MCX sits behind Akamai's WAF, which TLS-fingerprints plain HTTP clients
(reqwest included) and returns 403 / challenge pages. The only reliable way to
fetch MCX data is through `curl_cffi`, which impersonates a real browser TLS
fingerprint. This module provides that fallback implementation for
`get_mcx_bhavcopy`.

Endpoint (discovered from the site's own BhavCopy.js):
    GET https://www.mcxindia.com/market-data/bhavcopy/GetDateWiseBhavCopy
        ?InstrumentName=ALL&fromDate=dd/mm/yyyy

The current day's bhavcopy is also preloaded on the bhavcopy page, so the page
must be visited first to establish cookies before the JSON endpoint responds.
"""

import datetime
import time
from typing import Any, List, Optional

from .financeindia import DataError, FinanceError, NetworkError, StatusCodeError

_MCX_BASE = "https://www.mcxindia.com"
_MCX_BHAVCOPY_PAGE = f"{_MCX_BASE}/market-data/bhavcopy"
_MCX_DATE_WISE_URL = f"{_MCX_BASE}/market-data/bhavcopy/GetDateWiseBhavCopy"

_MCX_JSON_HEADERS = {
    "Accept": "application/json, text/javascript, */*; q=0.01",
    "X-Requested-With": "XMLHttpRequest",
    "Referer": _MCX_BHAVCOPY_PAGE,
}

# Formats accepted by the Rust `parse_date_robust` helper.
_DATE_FORMATS = (
    "%d-%m-%Y",
    "%Y-%m-%d",
    "%d%m%Y",
    "%Y%m%d",
    "%d-%b-%Y",
    "%d%b%Y",
)

_IMPERSONATE = "chrome124"
_TIMEOUT = 30
_ATTEMPTS = 3
_WARMUP_DELAY = 1.0
_RETRY_DELAY = 2.0


def _parse_date(date_str: str) -> str:
    clean = date_str.replace("/", "-").replace("\\", "-")
    for fmt in _DATE_FORMATS:
        try:
            return datetime.datetime.strptime(clean, fmt).date().strftime("%d/%m/%Y")
        except ValueError:
            continue
    raise DataError(
        "Invalid date format: '{}'. Supported formats include DD-MM-YYYY, "
        "YYYY-MM-DD, DD-Mon-YYYY.".format(date_str)
    )


def fetch_mcx_bhavcopy(date: str, instrument: str = "ALL") -> List[dict]:
    """Fetch the MCX bhavcopy for a date via the curl_cffi fallback.

    Returns a list of row dicts (one per contract) with keys such as Date,
    Symbol, InstrumentName, ExpiryDate, StrikePrice, OptionType, Open, High,
    Low, Close, PreviousClose, Volume, Value, OpenInterest.
    """
    from_date = _parse_date(date)

    try:
        from curl_cffi import requests as curl_requests
    except ImportError:
        raise DataError(
            "get_mcx_bhavcopy requires the optional 'curl_cffi' package because "
            "MCX's Akamai firewall blocks plain HTTP clients (including reqwest). "
            "Install it with 'pip install curl_cffi'."
        )

    last_error: Optional[FinanceError] = None
    for attempt in range(_ATTEMPTS):
        try:
            session = curl_requests.Session(impersonate=_IMPERSONATE, timeout=_TIMEOUT)

            warmup = session.get(_MCX_BHAVCOPY_PAGE, timeout=_TIMEOUT)
            if warmup.status_code != 200:
                raise NetworkError(
                    "MCX bhavcopy page returned HTTP {} (expected 200)".format(
                        warmup.status_code
                    )
                )
            time.sleep(_WARMUP_DELAY)

            response = session.get(
                _MCX_DATE_WISE_URL,
                params={"InstrumentName": instrument, "fromDate": from_date},
                headers=_MCX_JSON_HEADERS,
                timeout=_TIMEOUT,
            )
            if response.status_code != 200:
                raise StatusCodeError(
                    response.status_code,
                    "MCX GetDateWiseBhavCopy for date {}".format(from_date),
                )

            content_type = response.headers.get("content-type", "")
            if not content_type.startswith("application/json"):
                raise NetworkError(
                    "MCX returned a non-JSON response (content-type: '{}'); "
                    "this is usually an Akamai challenge page. Retrying.".format(
                        content_type
                    )
                )

            payload = response.json()
            if not payload.get("IsSuccess"):
                message = payload.get("Message") or "MCX bhavcopy request failed"
                raise DataError(message)

            rows = payload.get("Data")
            if rows is None:
                raise DataError("MCX bhavcopy returned no data")
            return rows

        except FinanceError as exc:
            last_error = exc
        except Exception as exc:  # curl_cffi transport errors
            last_error = NetworkError(
                "MCX bhavcopy fetch failed: {}: {}".format(type(exc).__name__, exc)
            )

        if attempt < _ATTEMPTS - 1:
            time.sleep(_RETRY_DELAY)

    raise last_error or NetworkError("MCX bhavcopy fetch failed")
