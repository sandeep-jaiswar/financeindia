use crate::common::{fetch_bytes, parse_date_robust};
use crate::error::{FinanceError, FinanceResult};
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use std::collections::HashMap;

/// Fetches financial results metadata for a security.
pub async fn financial_results(
    client: &Client,
    symbol: &str,
    from_date: &str,
    to_date: &str,
    period: &str,
) -> FinanceResult<Bytes> {
    crate::common::validate_string_param(symbol, "symbol", 20)?;
    crate::common::validate_period(period)?;
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

/// Fetches upcoming corporate actions (dividends, splits, etc.).
pub async fn corporate_actions(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/corporates-corporateActions?index=equities";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Downloads and parses an XBRL file from a trusted NSE domain into a flat JSON structure.
///
/// # Limitations
/// The output is a flat `HashMap<tag_name, Vec<{value, attrs}>>`. XBRL document structure
/// (contexts, periods, units, dimensions) is not preserved. Use this for quick fact extraction
/// only; for full XBRL semantics use a dedicated XBRL parser.
///
/// # Security
/// Only HTTPS URLs pointing to `*.nseindia.com` or `nseindia.com` are accepted (SSRF guard).
pub async fn parse_xbrl_data(client: &Client, xbrl_url: &str) -> FinanceResult<Bytes> {
    let url = url::Url::parse(xbrl_url).map_err(FinanceError::UrlParse)?;

    if url.scheme() != "https" {
        return Err(FinanceError::Runtime(
            "Only HTTPS URLs are allowed".to_string(),
        ));
    }

    let host = url
        .host_str()
        .ok_or_else(|| FinanceError::Runtime("URL has no host".to_string()))?;

    if !(host.ends_with(".nseindia.com") || host == "nseindia.com") {
        return Err(FinanceError::Runtime(
            "URL host must be a trusted NSE domain".to_string(),
        ));
    }

    let xml_bytes = fetch_bytes(client, xbrl_url, Some("https://www.nseindia.com/")).await?;
    let xml_str = String::from_utf8(xml_bytes.to_vec())
        .map_err(|e| FinanceError::Runtime(format!("UTF-8 error in XBRL response: {}", e)))?;

    parse_xbrl_xml(&xml_str)
}

fn parse_xbrl_xml(xml_str: &str) -> FinanceResult<Bytes> {
    if xml_str.to_uppercase().contains("<!DOCTYPE") {
        return Err(FinanceError::Runtime(
            "DOCTYPE declarations are not allowed in XBRL documents (XXE prevention)".to_string(),
        ));
    }

    let mut reader = Reader::from_str(xml_str);
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = true;

    let mut results: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut buf = Vec::new();
    // Keep each element's tag and attributes together across nested elements.
    let mut tag_stack: Vec<(String, HashMap<String, String>)> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_string();
                let mut attrs = HashMap::new();
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref())
                        .unwrap_or("")
                        .to_string();
                    let value = String::from_utf8_lossy(&attr.value).into_owned();
                    attrs.insert(key, value);
                }
                tag_stack.push((name, attrs));
            }
            Ok(Event::Text(e)) => {
                if let Some((tag, attrs)) = tag_stack.last() {
                    if let Ok(unescaped) = e.unescape() {
                        let text = unescaped.trim().to_string();
                        if !text.is_empty() {
                            let mut fact = serde_json::Map::new();
                            fact.insert("value".to_string(), serde_json::Value::String(text));
                            if !attrs.is_empty() {
                                let attrs_value = serde_json::to_value(attrs).map_err(|e| {
                                    FinanceError::Runtime(format!(
                                        "Attribute serialisation error: {}",
                                        e
                                    ))
                                })?;
                                fact.insert("attrs".to_string(), attrs_value);
                            }
                            results
                                .entry(tag.clone())
                                .or_default()
                                .push(serde_json::Value::Object(fact));
                        }
                    }
                    // Malformed/escaped text is silently skipped.
                }
            }
            Ok(Event::End(_)) => {
                tag_stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(FinanceError::Xml(e)),
            _ => (),
        }
        buf.clear();
    }

    serde_json::to_vec(&results)
        .map(Bytes::from)
        .map_err(|e| FinanceError::Runtime(format!("JSON serialisation error: {}", e)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parent_attributes_survive_empty_children() {
        for child in [
            "<child/>",
            "<child contextRef=\"child\"/>",
            "<child></child>",
            "<child contextRef=\"child\"></child>",
        ] {
            let xml = format!(
                "<root><parent contextRef=\"parent\">before{child}after</parent>\
                 <sibling contextRef=\"sibling\">next</sibling></root>"
            );
            let result: serde_json::Value =
                serde_json::from_slice(&parse_xbrl_xml(&xml).unwrap()).unwrap();
            assert_eq!(
                result,
                json!({
                    "parent": [
                        {"value": "before", "attrs": {"contextRef": "parent"}},
                        {"value": "after", "attrs": {"contextRef": "parent"}}
                    ],
                    "sibling": [{"value": "next", "attrs": {"contextRef": "sibling"}}]
                }),
                "child: {child}"
            );
        }
    }

    #[test]
    fn parent_without_attributes_does_not_inherit_child_attributes() {
        let result: serde_json::Value = serde_json::from_slice(
            &parse_xbrl_xml("<parent><child contextRef=\"child\"/>after</parent>").unwrap(),
        )
        .unwrap();
        assert_eq!(result, json!({"parent": [{"value": "after"}]}));
    }

    #[test]
    fn nested_elements_keep_their_own_attributes() {
        let xml = r#"<parent contextRef="parent">before
            <child contextRef="child">child before
                <grandchild unitRef="units">42</grandchild>
                child after
            </child>
            after
        </parent>"#;
        let result: serde_json::Value =
            serde_json::from_slice(&parse_xbrl_xml(xml).unwrap()).unwrap();
        assert_eq!(
            result,
            json!({
                "parent": [
                    {"value": "before", "attrs": {"contextRef": "parent"}},
                    {"value": "after", "attrs": {"contextRef": "parent"}}
                ],
                "child": [
                    {"value": "child before", "attrs": {"contextRef": "child"}},
                    {"value": "child after", "attrs": {"contextRef": "child"}}
                ],
                "grandchild": [{"value": "42", "attrs": {"unitRef": "units"}}]
            })
        );
    }
}
