use crate::common::fetch_text;
use pyo3::prelude::*;
use reqwest::blocking::Client;

/// Fetches Additional Surveillance Measure (ASM) stocks.
pub fn asm_stocks(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/reportASM";
    fetch_text(client, url, Some("https://www.nseindia.com/reports/asm"))
}

/// Fetches Graded Surveillance Measure (GSM) stocks.
pub fn gsm_stocks(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/reportGSM";
    fetch_text(client, url, Some("https://www.nseindia.com/reports/gsm"))
}
