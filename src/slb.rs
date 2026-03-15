use crate::common::{fetch_bytes, parse_date_robust};
use pyo3::prelude::*;
use reqwest::Client;

/// Fetches SLB Bhavcopy (DAT).
pub async fn slb_bhavcopy(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/slbs/bhavcopy/SLBM_BC_09032026.DAT
    let url = format!(
        "https://nsearchives.nseindia.com/archives/slbs/bhavcopy/SLBM_BC_{}.DAT",
        d.format("%d%m%Y")
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
    .await
}

/// Fetches SLB eligible securities suggestions.
pub async fn slb_eligible(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/quote/suggest/equity/slb";
    fetch_bytes(
        client,
        url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
    .await
}

/// Fetches SLB live analysis/open positions for a specific series.
pub async fn live_analysis_slb(client: &Client, series: &str) -> PyResult<bytes::Bytes> {
    let encoded_series = percent_encoding::utf8_percent_encode(series, percent_encoding::NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/live-analysis-slb?series={}",
        encoded_series
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
    .await
}

/// Fetches SLB series master (available months).
pub async fn slb_series_master(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/live-analysis-slb-series-master";
    fetch_bytes(
        client,
        url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
    .await
}
