use pyo3::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::io::Read;
use zip::ZipArchive;
use crate::common::{parse_date_robust, fetch_text};

/// Fetches the Equity Bhavcopy (UDiFF format) for a given date.
pub fn bhav_copy_equities(client: &Client, date: &str) -> PyResult<String> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/content/cm/BhavCopy_NSE_CM_0_0_0_{}_F_0000.csv.zip",
        d.format("%Y%m%d")
    );
    
    let response = client.get(&url)
        .header(REFERER, "https://www.nseindia.com/all-reports")
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

/// Fetches historical price and volume data for a given security.
pub fn price_volume_data(client: &Client, symbol: &str, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/generateSecurityWiseHistoricalData?from={}&to={}&symbol={}&type=priceVolume&series=ALL&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y"), encoded_symbol
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/report-detail/eq_security"))
}

/// Fetches bulk deal data for a date range.
pub fn bulk_deal_data(client: &Client, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=bulk_deals&from={}&to={}&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y")
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/report-detail/display-bulk-and-block-deals"))
}

/// Fetches block deals data for a date range.
pub fn block_deals_data(client: &Client, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=block_deals&from={}&to={}&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y")
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/report-detail/display-bulk-and-block-deals"))
}

/// Fetches short selling data for a date range.
pub fn short_selling_data(client: &Client, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=short_selling&from={}&to={}&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y")
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/report-detail/display-bulk-and-block-deals"))
}

/// Fetches advances and declines data.
pub fn advances_declines(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/marketStatus"; 
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}



/// Fetches monthly settlement statistics.
pub fn monthly_settlement_stats(client: &Client, fin_year: &str) -> PyResult<String> {
    // fin_year format: YYYY-YYYY
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/monthly-sett-stats-data?finYear={}",
        fin_year
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches 52 week high/low data.
pub fn fifty_two_week_high_low(client: &Client, mode: &str) -> PyResult<String> {
    let url = if mode == "low" {
        "https://www.nseindia.com/api/live-analysis-data-52weeklowstock"
    } else {
        "https://www.nseindia.com/api/live-analysis-data-52weekhighstock"
    };
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches most active securities.
pub fn most_active(client: &Client, mode: &str) -> PyResult<String> {
    let url = format!("https://www.nseindia.com/api/live-analysis-most-active-securities?index={}", mode);
    fetch_text(client, &url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches top gainers.
pub fn top_gainers(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/live-analysis-variations?index=gainers";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches top losers.
pub fn top_losers(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/live-analysis-variations?index=loosers";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches deliverable position data for a given security.
pub fn deliverable_position_data(client: &Client, symbol: &str, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/generateSecurityWiseHistoricalData?from={}&to={}&symbol={}&type=deliverable&series=ALL&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y"), encoded_symbol
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/report-detail/eq_security"))
}

/// Fetches the list of all active equities.
pub fn equity_list(client: &Client) -> PyResult<String> {
    let url = "https://archives.nseindia.com/content/equities/EQUITY_L.csv";
    fetch_text(client, url, Some("https://www.nseindia.com"))
}

/// Fetches a detailed quote for an equity symbol.
pub fn equity_quote(client: &Client, symbol: &str) -> PyResult<String> {
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.nseindia.com/api/quote-equity?symbol={}", encoded_symbol);
    fetch_text(client, &url, Some(&format!("https://www.nseindia.com/get-quotes/equity?symbol={}", encoded_symbol)))
}

