use pyo3::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use std::io::Read;
use zip::ZipArchive;
use crate::common::{parse_date_robust, fetch_text};

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
    
    let response = client.get(&url)
        .header(REFERER, "https://www.nseindia.com/all-reports-derivatives")
        .send()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Network error: {}", e)))?;

    let checked = response.error_for_status()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("HTTP error: {}", e)))?;

    let bytes = checked.bytes()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to read response bytes: {}", e)))?;

    let reader = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to open zip archive: {}", e)))?;

    if archive.len() == 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Zip archive is empty"));
    }

    let mut file = archive.by_index(0)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get file from zip: {}", e)))?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to read file content: {}", e)))?;

    Ok(content)
}

/// Fetches the option chain for a given symbol.
pub fn option_chain(client: &Client, symbol: &str, is_index: bool) -> PyResult<String> {
    let api_type = if is_index { "indices" } else { "equities" };
    let url = format!("https://www.nseindia.com/api/option-chain-{}?symbol={}", api_type, symbol);
    fetch_text(client, &url, Some("https://www.nseindia.com/option-chain"))
}

/// Fetches FO security ban list.
pub fn fo_sec_ban(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/equity-stockIndices?index=SECURITIES%20IN%20F%26O%20BAN%20PERIOD";
    fetch_text(client, url, Some("https://www.nseindia.com/market-data/live-equity-market"))
}



/// Fetches SPAN margins (zip file).
pub fn span_margins(client: &Client, date: &str) -> PyResult<String> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/nsccl/span/nsccl.20260309.i1.zip
    let url = format!(
        "https://nsearchives.nseindia.com/archives/nsccl/span/nsccl.{}.i1.zip",
        d.format("%Y%m%d")
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/all-reports-derivatives"))
}






