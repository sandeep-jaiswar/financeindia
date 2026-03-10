use pyo3::prelude::*;
use reqwest::blocking::Client;
use crate::common::{parse_date_robust, fetch_text};

/// Fetches FII/DII trading activity (React API).
pub fn fii_dii_activity(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/fiidiiTradeReact";
    fetch_text(client, url, Some("https://www.nseindia.com/reports/fii-dii"))
}

/// Fetches detailed FII statistics (.xls).
pub fn fii_stats(client: &Client, date: &str) -> PyResult<Vec<u8>> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/content/fo/fii_stats_09-Mar-2026.xls
    let url = format!(
        "https://nsearchives.nseindia.com/content/fo/fii_stats_{}.xls",
        d.format("%d-%b-%Y")
    );
    let bytes = crate::common::fetch_bytes(client, &url, Some("https://www.nseindia.com/all-reports"))?;
    Ok(bytes.to_vec())
}

/// Fetches market turnover.
pub fn market_turnover(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/marketStatus";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches market holidays.
pub fn holidays(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/holiday-master?type=trading";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Returns the current market status.
pub fn market_status(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/marketStatus";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}
