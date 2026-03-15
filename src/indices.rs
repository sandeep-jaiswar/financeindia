use crate::common::{fetch_bytes, parse_date_robust};
use crate::error::FinanceResult;
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::Client;

/// Fetches a list of all NSE market indices.
pub async fn all_indices(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/allIndices";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches constituent stocks for a given index (e.g. `"NIFTY 50"`).
pub async fn index_constituents(client: &Client, index: &str) -> FinanceResult<Bytes> {
    let encoded_index = utf8_percent_encode(index, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/equity-stockIndices?index={}",
        encoded_index
    );
    fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches historical OHLCV data for a specific index.
pub async fn index_history(
    client: &Client,
    index: &str,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_index = utf8_percent_encode(index, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/indicesHistory?indexType={}&from={}&to={}",
        encoded_index,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-historical-index-data"),
    )
    .await
}

/// Fetches P/E, P/B, and Dividend Yield for a specific index.
pub async fn index_yield(
    client: &Client,
    index: &str,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_index = utf8_percent_encode(index, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/indicesYield?indexType={}&from={}&to={}",
        encoded_index,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-yield"),
    )
    .await
}

/// Fetches India VIX historical data.
pub async fn india_vix_history(
    client: &Client,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/vixHistory?from={}&to={}",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-historical-vix"),
    )
    .await
}

/// Fetches Total Returns Index (TRI) historical values.
///
/// Delegates to `index_history` with the `tri=true` query parameter.
pub async fn total_returns_index(
    client: &Client,
    index: &str,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_index = utf8_percent_encode(index, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/indicesHistory?indexType={}&from={}&to={}&tri=true",
        encoded_index,
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/reports-indices-historical-index-data"),
    )
    .await
}
