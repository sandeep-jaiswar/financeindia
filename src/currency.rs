use crate::common::{
    fetch_bytes, parse_date_robust, read_first_text_file_from_zip,
};
use crate::error::FinanceResult;
use bytes::Bytes;
use reqwest::Client;

/// Fetch Currency Derivatives Bhavcopy for a given date.
///
/// NSE discontinued the per-date CSV (`bhavcopy_cde_*.csv`) in July 2024 and now
/// publishes a UDiFF common bhavcopy zip under `/archives/cd/bhav/`. The first
/// CSV entry of the archive is returned.
pub async fn currency_bhavcopy(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/archives/cd/bhav/BhavCopy_NSE_CD_0_0_0_{}_F_0000.csv.zip",
        d.format("%Y%m%d")
    );
    let bytes = fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await?;
    read_first_text_file_from_zip(bytes)
}

/// Fetch live market data for Currency Derivatives.
///
/// NSE replaced the old `liveCurrency-Market` endpoint with
/// `liveCurrency-derivatives` (the market-watch feed).
pub async fn live_currency_market(client: &Client) -> FinanceResult<Bytes> {
    let url =
        "https://www.nseindia.com/api/liveCurrency-derivatives?index=live_market_currency_spread";
    fetch_bytes(
        client,
        url,
        Some("https://www.nseindia.com/market-data/currency-derivatives"),
    )
    .await
}
