use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use reqwest::blocking::Client;
use reqwest::header::{REFERER, ACCEPT};
use chrono::NaiveDate;
use std::io::Read;
use zip::ZipArchive;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

/// Internal helper to parse dates from various common formats.
fn parse_date_robust(date: &str) -> PyResult<NaiveDate> {
    let clean = date.replace("-", "").replace("/", "");
    
    // Try formats in order of likelihood
    let formats = ["%d%m%Y", "%Y%m%d", "%d-%m-%Y", "%Y-%m-%d"];
    
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(date, fmt) {
            return Ok(d);
        }
    }
    
    // Fallback for cleaned strings
    if clean.len() == 8 {
        if clean.starts_with("20") || clean.starts_with("19") {
            if let Ok(d) = NaiveDate::parse_from_str(&clean, "%Y%m%d") {
                return Ok(d);
            }
        } else {
            if let Ok(d) = NaiveDate::parse_from_str(&clean, "%d%m%Y") {
                return Ok(d);
            }
        }
    }

    Err(PyErr::new::<PyRuntimeError, _>(format!(
        "Invalid date format: '{}'. Supported: DD-MM-YYYY, DDMMYYYY, YYYY-MM-DD, YYYYMMDD.", 
        date
    )))
}

/// Internal helper to execute a GET request and handle errors consistently.
fn fetch_text(client: &Client, url: &str, referer: Option<&str>) -> PyResult<String> {
    let mut rb = client.get(url).header(ACCEPT, "*/*");
    if let Some(r) = referer {
        rb = rb.header(REFERER, r);
    }
    
    let response = rb.send()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Network error: {}", e)))?;
        
    let checked = response.error_for_status()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("HTTP error: {}", e)))?;
        
    checked.text()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to read response body: {}", e)))
}

/// Fetches historical price and volume data for a given security.
pub fn price_volume_data(client: &Client, symbol: &str, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    
    if from > to {
        return Err(PyErr::new::<PyRuntimeError, _>("from_date cannot be after to_date"));
    }

    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/generateSecurityWiseHistoricalData?from={}&to={}&symbol={}&type=priceVolume&series=ALL&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y"), encoded_symbol
    );
    
    fetch_text(client, &url, Some("https://nsewebsite-staging.nseindia.com/report-detail/eq_security"))
}

/// Fetches deliverable position data for a given security.
pub fn deliverable_position_data(client: &Client, symbol: &str, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    
    if from > to {
        return Err(PyErr::new::<PyRuntimeError, _>("from_date cannot be after to_date"));
    }

    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/generateSecurityWiseHistoricalData?from={}&to={}&symbol={}&type=deliverable&series=ALL&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y"), encoded_symbol
    );
    
    fetch_text(client, &url, Some("https://nsewebsite-staging.nseindia.com/report-detail/eq_security"))
}

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
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Network error: {}", e)))?;

    let checked = response.error_for_status()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("HTTP error: {}", e)))?;

    let bytes = checked.bytes()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to read response bytes: {}", e)))?;

    let reader = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to open zip archive: {}", e)))?;

    if archive.len() == 0 {
        return Err(PyErr::new::<PyRuntimeError, _>("Zip archive is empty"));
    }

    let mut file = archive.by_index(0)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to get file from zip: {}", e)))?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to read file content: {}", e)))?;

    Ok(content)
}

/// Fetches the list of all active equities.
pub fn equity_list(client: &Client) -> PyResult<String> {
    let url = "https://archives.nseindia.com/content/equities/EQUITY_L.csv";
    fetch_text(client, url, Some("https://nsewebsite-staging.nseindia.com"))
}

/// Fetches bulk deal data for a date range.
pub fn bulk_deal_data(client: &Client, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    
    if from > to {
        return Err(PyErr::new::<PyRuntimeError, _>("from_date cannot be after to_date"));
    }
    
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=bulk_deals&from={}&to={}&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y")
    );
    
    fetch_text(client, &url, Some("https://nsewebsite-staging.nseindia.com"))
}

/// Fetches block deals data for a date range.
pub fn block_deals_data(client: &Client, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    
    if from > to {
        return Err(PyErr::new::<PyRuntimeError, _>("from_date cannot be after to_date"));
    }
    
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=block_deals&from={}&to={}&csv=true",
        from.format("%d-%m-%Y"), to.format("%d-%m-%Y")
    );
    
    fetch_text(client, &url, Some("https://nsewebsite-staging.nseindia.com"))
}

/// Fetches the list of Nifty 50 constituent stocks.
pub fn nifty50_equity_list(client: &Client) -> PyResult<String> {
    let url = "https://nsearchives.nseindia.com/content/indices/ind_nifty50list.csv";
    fetch_text(client, url, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_parsing() {
        assert_eq!(parse_date_robust("05032026").unwrap().format("%Y%m%d").to_string(), "20260305");
        assert_eq!(parse_date_robust("20260305").unwrap().format("%Y%m%d").to_string(), "20260305");
        assert_eq!(parse_date_robust("05-03-2026").unwrap().format("%Y%m%d").to_string(), "20260305");
        assert_eq!(parse_date_robust("2026-03-05").unwrap().format("%Y%m%d").to_string(), "20260305");
        assert!(parse_date_robust("invalid").is_err());
    }
}
