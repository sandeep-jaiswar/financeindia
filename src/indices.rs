use crate::common::{fetch_text, parse_date_robust};
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use pyo3::prelude::*;
use reqwest::blocking::Client;

/// Fetches a list of all NSE market indices.
pub fn all_indices(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/allIndices";
    fetch_text(client, url, Some(crate::common::NSE_ALL_REPORTS_URL))
}

/// Fetches constituent stocks for a given index.
pub fn index_constituents(client: &Client, index: &str) -> PyResult<bytes::Bytes> {
    let encoded_index = percent_encode(index.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/equity-stockIndices?index={}",
        encoded_index
    );
    fetch_text(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL))
}

/// Fetches historical index data (OHLCV).
pub fn index_history(
    client: &Client,
    index: &str,
    from_date: &str,
    to_date: &str,
) -> PyResult<bytes::Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_index = percent_encode(index.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/indicesHistory?indexType={}&from={}&to={}",
        encoded_index,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_text(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-historical-index-data"),
    )
}

/// Fetches P/E, P/B and Div Yield for a given index.
pub fn index_yield(
    client: &Client,
    index: &str,
    from_date: &str,
    to_date: &str,
) -> PyResult<bytes::Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_index = percent_encode(index.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/indicesYield?indexType={}&from={}&to={}",
        encoded_index,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_text(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-yield"),
    )
}

/// Fetches India VIX historical data.
pub fn india_vix_history(
    client: &Client,
    from_date: &str,
    to_date: &str,
) -> PyResult<bytes::Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/vixHistory?from={}&to={}",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_text(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-historical-vix"),
    )
}

/// Fetches Total Returns Index (TRI) values.
pub fn total_returns_index(
    client: &Client,
    index: &str,
    from_date: &str,
    to_date: &str,
) -> PyResult<bytes::Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_index = percent_encode(index.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/indicesHistory?indexType={}&from={}&to={}&tri=true",
        encoded_index,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_text(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-historical-index-data"),
    )
}
