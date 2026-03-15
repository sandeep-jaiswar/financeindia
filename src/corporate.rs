use crate::common::{fetch_bytes, parse_date_robust};
use crate::error::{FinanceError, FinanceResult};
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use std::collections::HashMap;

/// Fetches financial results metadata.
pub async fn financial_results(
    client: &Client,
    symbol: &str,
    from_date: &str,
    to_date: &str,
    period: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let encoded_period = percent_encode(period.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/corporates-financial-results?index=equities&symbol={}&from_date={}&to_date={}&period={}",
        encoded_symbol,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT),
        encoded_period
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/companies-listing/corporate-filings-financial-results"),
    )
    .await
}

/// Fetches upcoming corporate actions.
pub async fn corporate_actions(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/corporates-corporateActions?index=equities";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Downloads and parses an XBRL file into JSON.
pub async fn parse_xbrl_data(client: &Client, xbrl_url: &str) -> FinanceResult<Bytes> {
    // SSRF Validation
    let url = reqwest::Url::parse(xbrl_url)
        .map_err(|e| FinanceError::Runtime(format!("Invalid URL: {}", e)))?;

    if url.scheme() != "https" {
        return Err(FinanceError::Runtime("Only HTTPS URLs are allowed".to_string()));
    }

    let host = url.host_str().unwrap_or_default();
    if !host.ends_with(".nseindia.com") && host != "nseindia.com" {
        return Err(FinanceError::Runtime(
            "URL host must be a trusted NSE domain".to_string(),
        ));
    }

    let xml_bytes = fetch_bytes(client, xbrl_url, Some("https://www.nseindia.com/")).await?;
    let xml_str = String::from_utf8(xml_bytes.to_vec())
        .map_err(|e| FinanceError::Runtime(format!("UTF-8 Error: {}", e)))?;
    let mut reader = Reader::from_str(&xml_str);
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
                    if let Ok(unescaped) = e.unescape() {
                        let text = unescaped.to_string();
                        if !text.is_empty() {
                            let mut fact = serde_json::Map::new();
                            fact.insert("value".to_string(), serde_json::Value::String(text));
                            if !current_attrs.is_empty() {
                                let attrs_value =
                                    serde_json::to_value(&current_attrs).map_err(|e| {
                                        FinanceError::Runtime(format!(
                                            "Attr serialization error: {}",
                                            e
                                        ))
                                    })?;
                                fact.insert("attrs".to_string(), attrs_value);
                            }
                            results
                                .entry(tag.clone())
                                .or_insert_with(Vec::new)
                                .push(serde_json::Value::Object(fact));
                        }
                    } else {
                        // For now, we skip malformed text as per desired robust behavior
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
                current_attrs.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(FinanceError::Runtime(format!("XML Error: {}", e))),
            _ => (),
        }
        buf.clear();
    }
    serde_json::to_vec(&results)
        .map(Bytes::from)
        .map_err(|e| FinanceError::Runtime(format!("JSON Serialization Error: {}", e)))
}

/// Fetches insider trades (PIT) data for a given date range.
pub async fn insider_trades(
    client: &Client,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/corporates-pit?index=equities&from_date={}&to_date={}",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/companies-listing/corporate-filings-insider-trading"),
    )
    .await
}
