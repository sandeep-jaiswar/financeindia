use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use reqwest::blocking::Client;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use crate::common::{parse_date_robust, fetch_text};

/// Fetches financial results metadata.
pub fn financial_results(client: &Client, symbol: &str, from_date: &str, to_date: &str, period: &str) -> PyResult<String> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let encoded_period = percent_encode(period.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/corporates-financial-results?index=equities&symbol={}&from_date={}&to_date={}&period={}",
        encoded_symbol, from.format("%d-%m-%Y"), to.format("%d-%m-%Y"), encoded_period
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/companies-listing/corporate-filings-financial-results"))
}

/// Fetches upcoming corporate actions.
pub fn corporate_actions(client: &Client) -> PyResult<String> {
    let url = "https://www.nseindia.com/api/corporates-corporateActions?index=equities";
    fetch_text(client, url, Some("https://www.nseindia.com/all-reports"))
}

/// Downloads and parses an XBRL file into JSON.
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
                        results.entry(tag.clone()).or_insert_with(Vec::new).push(serde_json::Value::Object(fact));
                    }
                }
            }
            Ok(Event::End(_)) => { current_tag = None; current_attrs.clear(); }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PyErr::new::<PyRuntimeError, _>(format!("XML Error: {}", e))),
            _ => (),
        }
        buf.clear();
    }
    serde_json::to_string(&results).map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON Serialization Error: {}", e)))
}
