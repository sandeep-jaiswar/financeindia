use crate::common::{fetch_bytes, parse_date_robust};
use crate::error::FinanceResult;
use bytes::Bytes;
use reqwest::Client;

/// Fetch Commodities Derivatives Bhavcopy for a given date.
pub async fn nse_commodities_bhavcopy(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/content/nsccl/bhavcopy_cbo_{}.csv",
        d.format("%d%m%Y")
    );
    fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetch live market data for Commodities.
pub async fn nse_live_commodities_market(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/liveCommodity-Market";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetch MCX Bhavcopy.
pub async fn mcx_bhavcopy(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://www.mcxindia.com/en/bhavcopy/bhavcopy.aspx?date={}",
        d.format("%d/%m/%Y")
    );
    fetch_bytes(client, &url, None).await
}
