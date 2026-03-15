use crate::common::{fetch_bytes, parse_date_robust, read_first_text_file_from_zip};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use pyo3::prelude::*;
use reqwest::Client;

/// Fetches the F&O Bhavcopy for a given date.
pub async fn bhav_copy_derivatives(
    client: &Client,
    date: &str,
    segment: &str,
) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    let (prefix, seg_code) = match segment.to_uppercase().as_str() {
        "FO" | "F&O" => ("FO", "FO"),
        "CO" | "COMMODITY" => ("CO", "CO"),
        "CD" | "CURRENCY" => ("CD", "CD"),
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Invalid segment. Use FO, CO, or CD.",
            ));
        }
    };

    let url = format!(
        "https://nsearchives.nseindia.com/content/{}/BhavCopy_NSE_{}_0_0_0_{}_F_0000.csv.zip",
        seg_code.to_lowercase(),
        prefix,
        d.format("%Y%m%d")
    );

    let bytes = fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/all-reports-derivatives"),
    )
    .await?;
    read_first_text_file_from_zip(bytes)
}

/// Fetches the option chain for a given symbol.
pub async fn option_chain(client: &Client, symbol: &str, is_index: bool) -> PyResult<bytes::Bytes> {
    let api_type = if is_index { "indices" } else { "equities" };
    let encoded_symbol = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/option-chain-{}?symbol={}",
        api_type, encoded_symbol
    );
    fetch_bytes(client, &url, Some("https://www.nseindia.com/option-chain")).await
}

/// Fetches FO security ban list.
pub async fn fo_sec_ban(client: &Client) -> PyResult<bytes::Bytes> {
    let url = "https://www.nseindia.com/api/equity-stockIndices?index=SECURITIES%20IN%20F%26O%20BAN%20PERIOD";
    fetch_bytes(
        client,
        url,
        Some("https://www.nseindia.com/market-data/live-equity-market"),
    )
    .await
}

/// Fetches SPAN margins (zip file containing a CSV/DAT).
pub async fn span_margins(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/nsccl/span/nsccl.20260309.i1.zip
    let url = format!(
        "https://nsearchives.nseindia.com/archives/nsccl/span/nsccl.{}.i1.zip",
        d.format("%Y%m%d")
    );

    let bytes = fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/all-reports-derivatives"),
    )
    .await?;
    read_first_text_file_from_zip(bytes)
}

/// Fetches FO security ban list as CSV for a given date.
pub async fn fo_sec_ban_csv(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/fo/sec_ban/fo_secban_11032026.csv
    let url = format!(
        "https://nsearchives.nseindia.com/archives/fo/sec_ban/fo_secban_{}.csv",
        d.format("%d%m%Y")
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/all-reports-derivatives"),
    )
    .await
}

/// Fetches participant wise trading volumes (CSV) for a given date.
pub async fn participant_volume(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/content/nsccl/fao_participant_vol_10032026.csv
    let url = format!(
        "https://nsearchives.nseindia.com/content/nsccl/fao_participant_vol_{}.csv",
        d.format("%d%m%Y")
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/all-reports-derivatives"),
    )
    .await
}

/// Fetches client wise OI limits (LST file) for a given date.
pub async fn oi_client_limits(client: &Client, date: &str) -> PyResult<bytes::Bytes> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/content/nsccl/oi_cli_limit_10-MAR-2026.lst
    let date_str = d.format("%d-%b-%Y").to_string().to_uppercase();
    let url = format!(
        "https://nsearchives.nseindia.com/content/nsccl/oi_cli_limit_{}.lst",
        date_str
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/all-reports-derivatives"),
    )
    .await
}
