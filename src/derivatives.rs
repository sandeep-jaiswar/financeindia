use pyo3::prelude::*;
use reqwest::blocking::Client;
use crate::common::{parse_date_robust, fetch_text, fetch_bytes, read_first_text_file_from_zip};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

/// Fetches the F&O Bhavcopy for a given date.
pub fn bhav_copy_derivatives(client: &Client, date: &str, segment: &str) -> PyResult<String> {
    let d = parse_date_robust(date)?;
    let (prefix, seg_code) = match segment.to_uppercase().as_str() {
        "FO" | "F&O" => ("FO", "FO"),
        "CO" | "COMMODITY" => ("CO", "CO"),
        "CD" | "CURRENCY" => ("CD", "CD"),
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid segment. Use FO, CO, or CD.")),
    };

    let url = format!(
        "https://nsearchives.nseindia.com/content/{}/BhavCopy_NSE_{}_0_0_0_{}_F_0000.csv.zip",
        seg_code.to_lowercase(), prefix, d.format("%Y%m%d")
    );
    
    let bytes = fetch_bytes(client, &url, Some("https://www.nseindia.com/all-reports-derivatives"))?;
    read_first_text_file_from_zip(bytes)
}

/// Fetches the option chain for a given symbol.
pub fn option_chain(client: &Client, symbol: &str, is_index: bool) -> PyResult<String> {
    let api_type = if is_index { "indices" } else { "equities" };
    let encoded_symbol = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/option-chain-{}?symbol={}",
        api_type, encoded_symbol
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/option-chain"))
}

/// Fetches FO security ban list.
pub fn fo_sec_ban(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/equity-stockIndices?index=SECURITIES%20IN%20F%26O%20BAN%20PERIOD";
    fetch_text(client, url, Some("https://www.nseindia.com/market-data/live-equity-market"))
}



/// Fetches SPAN margins (zip file containing a CSV/DAT).
pub fn span_margins(client: &Client, date: &str) -> PyResult<String> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/nsccl/span/nsccl.20260309.i1.zip
    let url = format!(
        "https://nsearchives.nseindia.com/archives/nsccl/span/nsccl.{}.i1.zip",
        d.format("%Y%m%d")
    );
    
    let bytes = fetch_bytes(client, &url, Some("https://www.nseindia.com/all-reports-derivatives"))?;
    read_first_text_file_from_zip(bytes)
}
