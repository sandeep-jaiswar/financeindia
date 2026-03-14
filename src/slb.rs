use crate::common::{fetch_text, parse_date_robust};
use pyo3::prelude::*;
use reqwest::blocking::Client;

/// Fetches SLB Bhavcopy (DAT).
pub fn slb_bhavcopy(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/slbs/bhavcopy/SLBM_BC_09032026.DAT
    let url = format!(
        "https://nsearchives.nseindia.com/archives/slbs/bhavcopy/SLBM_BC_{}.DAT",
        d.format("%d%m%Y")
    );
    fetch_text(
        client,
        &url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
}

/// Fetches SLB eligible securities suggestions.
pub fn slb_eligible(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/quote/suggest/equity/slb";
    fetch_text(
        client,
        url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
}

/// Fetches SLB live analysis/open positions for a specific series.
pub fn live_analysis_slb(client: &Client, series: &str) -> PyResult<bytes::Bytes> {
    let url = format!(
        "https://www.nseindia.com/api/live-analysis-slb?series={}",
        series
    );
    fetch_text(
        client,
        &url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
}

/// Fetches SLB series master (available months).
pub fn slb_series_master(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/live-analysis-slb-series-master";
    fetch_text(
        client,
        url,
        Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"),
    )
}
