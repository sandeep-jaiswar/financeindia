use crate::common::{fetch_bytes, parse_date_robust, read_first_text_file_from_zip};
use crate::error::FinanceResult;
use bytes::Bytes;
use reqwest::Client;

/// Fetch Commodities Derivatives Bhavcopy for a given date.
///
/// NSE discontinued the per-date CSV (`bhavcopy_cbo_*.csv`) in July 2024 and now
/// publishes a UDiFF common bhavcopy zip under `/content/com/`. The first CSV
/// entry of the archive is returned.
pub async fn nse_commodities_bhavcopy(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/content/com/BhavCopy_NSE_CO_0_0_0_{}_F_0000.csv.zip",
        d.format("%Y%m%d")
    );
    let bytes = fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await?;
    read_first_text_file_from_zip(bytes)
}

/// Fetch live market data for Commodities.
///
/// NSE replaced the old `liveCommodity-Market` endpoint with the
/// `commodity-futures` market-watch feed.
pub async fn nse_live_commodities_market(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/commodity-futures";
    fetch_bytes(
        client,
        url,
        Some("https://www.nseindia.com/market-data/commodity-derivatives"),
    )
    .await
}
