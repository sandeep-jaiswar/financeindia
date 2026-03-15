use crate::common::{fetch_bytes, parse_date_robust};
use pyo3::prelude::*;
use reqwest::Client;

/// Fetch Currency Derivatives Bhavcopy for a given date.
pub async fn currency_bhavcopy(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/content/nsccl/bhavcopy_cde_{}.csv",
        d.format("%d%m%Y")
    );
    fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetch live market data for Currency Derivatives.
pub async fn live_currency_market(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/liveCurrency-Market";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}
