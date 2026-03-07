use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use reqwest::blocking::Client;
use reqwest::header::{REFERER, ACCEPT};
use chrono::NaiveDate;
use std::io::Read;
use zip::ZipArchive;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;

/// Internal helper to parse dates from various common formats.
fn parse_date_robust(date: &str) -> PyResult<NaiveDate> {
    // Try formats in order of likelihood
    let formats = ["%d-%m-%Y", "%Y-%m-%d", "%d%m%Y", "%Y%m%d"];
    
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(date, fmt) {
            return Ok(d);
        }
    }
    
    // Fallback for cleaned strings if no delimiters matched
    let clean = date.replace("-", "").replace("/", "");
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

    Err(PyErr::new::<PyValueError, _>(format!(
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
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Network error while fetching {}: {}", url, e)))?;
        
    let checked = response.error_for_status()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("HTTP error {} for {}", e.status().unwrap_or_default(), url)))?;
        
    checked.text()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to read response body from {}: {}", url, e)))
}

/// Fetches historical price and volume data for a given security.
pub fn price_volume_data(client: &Client, symbol: &str, from_date: &str, to_date: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    
    if from > to {
        return Err(PyErr::new::<PyValueError, _>("from_date must be <= to_date"));
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
        return Err(PyErr::new::<PyValueError, _>("from_date must be <= to_date"));
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
        return Err(PyErr::new::<PyValueError, _>("from_date must be <= to_date"));
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
        return Err(PyErr::new::<PyValueError, _>("from_date must be <= to_date"));
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

/// Fetches all market indices.
pub fn all_indices(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/allIndices";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches constituents of a specific index.
pub fn index_constituents(client: &Client, index: &str) -> PyResult<String> {
    let encoded_index = percent_encode(index.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.nseindia.com/api/equity-stockIndices?index={}", encoded_index);
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

/// Fetches most active securities.
pub fn most_active(client: &Client, mode: &str) -> PyResult<String> {
    let url = format!("https://www.nseindia.com/api/live-analysis-most-active-securities?index={}", mode);
    fetch_text(client, &url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches a detailed quote for an equity symbol.
pub fn equity_quote(client: &Client, symbol: &str) -> PyResult<String> {
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.nseindia.com/api/quote-equity?symbol={}", encoded_symbol);
    fetch_text(client, &url, Some(&format!("https://www.nseindia.com/get-quotes/equity?symbol={}", encoded_symbol)))
}

/// Fetches option chain data.
pub fn option_chain(client: &Client, symbol: &str, is_index: bool) -> PyResult<String> {
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let api_type = if is_index { "indices" } else { "equities" };
    let url = format!("https://www.nseindia.com/api/option-chain-{}?symbol={}", api_type, encoded_symbol);
    fetch_text(client, &url, Some(&format!("https://www.nseindia.com/option-chain?symbol={}", encoded_symbol)))
}

/// Fetches market holidays.
pub fn holidays(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/holiday-master?type=trading";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches upcoming corporate actions.
pub fn corporate_actions(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/corporates-corporateActions?index=equities";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Fetches financial results metadata for a given security and period.
/// period: 'Quarterly', 'Annual', 'Half Yearly', etc.
pub fn financial_results(client: &Client, symbol: &str, from_date: &str, to_date: &str, period: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    
    if from > to {
        return Err(PyErr::new::<PyValueError, _>("from_date must be <= to_date"));
    }
    
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let encoded_period = percent_encode(period.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/corporates-financial-results?index=equities&symbol={}&from_date={}&to_date={}&period={}",
        encoded_symbol, from.format("%d-%m-%Y"), to.format("%d-%m-%Y"), encoded_period
    );
    
    fetch_text(client, &url, Some("https://www.nseindia.com/companies-listing/corporate-filings-financial-results"))
}

/// Downloads and parses an XBRL file into a comprehensive JSON format.
/// This ensures all columns/tags are captured without data loss.
pub fn parse_xbrl_data(client: &Client, xbrl_url: &str) -> PyResult<String> {
    // SSRF Validation
    let url = reqwest::Url::parse(xbrl_url)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid URL: {}", e)))?;
    
    if url.scheme() != "https" {
        return Err(PyErr::new::<PyValueError, _>("Only HTTPS URLs are allowed"));
    }
    
    let host = url.host_str().unwrap_or_default();
    if !host.ends_with(".nseindia.com") && host != "nseindia.com" {
        return Err(PyErr::new::<PyValueError, _>("URL host must be a trusted NSE domain"));
    }

    let xml_content = fetch_text(client, xbrl_url, Some("https://www.nseindia.com/"))?;
    
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);
    
    let mut results: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut buf = Vec::new();
    let mut current_tag: Option<String> = None;
    let mut current_attrs: HashMap<String, String> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                current_tag = Some(name.clone());
                
                // Extract attributes (like unitRef, contextRef)
                current_attrs.clear();
                for attr in e.attributes() {
                    if let Ok(a) = attr {
                        let key = String::from_utf8_lossy(a.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&a.value).to_string();
                        current_attrs.insert(key, value);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(ref tag) = current_tag {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if !text.is_empty() {
                        let mut fact = serde_json::Map::new();
                        fact.insert("value".to_string(), serde_json::Value::String(text));
                        if !current_attrs.is_empty() {
                            fact.insert("attrs".to_string(), serde_json::to_value(&current_attrs).unwrap_or_default());
                        }
                        
                        results.entry(tag.clone())
                            .or_insert_with(Vec::new)
                            .push(serde_json::Value::Object(fact));
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
                current_attrs.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PyErr::new::<PyRuntimeError, _>(format!("XML Error: {}", e))),
            _ => (),
        }
        buf.clear();
    }

    serde_json::to_string(&results)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON Serialization Error: {}", e)))
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
