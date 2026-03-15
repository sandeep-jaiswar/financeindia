use crate::common::{fetch_bytes, parse_date_robust, read_first_text_file_from_zip};
use crate::error::{FinanceError, FinanceResult};
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::Client;

/// Fetches the F&O Bhavcopy for a given date and segment.
///
/// `segment` must be one of: `"FO"`, `"F&O"`, `"CO"`, `"COMMODITY"`, `"CD"`, `"CURRENCY"`.
pub async fn bhav_copy_derivatives(
    client: &Client,
    date: &str,
    segment: &str,
) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    // Each arm carries both the URL path component and the archive prefix to avoid
    // a runtime `.to_lowercase()` allocation on a known static value.
    let (seg_lower, seg_upper) = match segment.to_uppercase().as_str() {
        "FO" | "F&O" => ("fo", "FO"),
        "CO" | "COMMODITY" => ("co", "CO"),
        "CD" | "CURRENCY" => ("cd", "CD"),
        _ => {
            return Err(FinanceError::Runtime(
                "Invalid segment. Use FO, CO, or CD.".to_string(),
            ));
        }
    };

    let url = format!(
        "https://nsearchives.nseindia.com/content/{}/BhavCopy_NSE_{}_0_0_0_{}_F_0000.csv.zip",
        seg_lower,
        seg_upper,
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
///
/// Set `is_index = true` for index option chains (e.g. NIFTY), `false` for equity chains.
pub async fn option_chain(client: &Client, symbol: &str, is_index: bool) -> FinanceResult<Bytes> {
    let api_type = if is_index { "indices" } else { "equities" };
    let encoded_symbol = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/option-chain-{}?symbol={}",
        api_type, encoded_symbol
    );
    fetch_bytes(client, &url, Some("https://www.nseindia.com/option-chain")).await
}

/// Fetches the live F&O security ban list.
pub async fn fo_sec_ban(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/equity-stockIndices?index=SECURITIES%20IN%20F%26O%20BAN%20PERIOD";
    fetch_bytes(
        client,
        url,
        Some("https://www.nseindia.com/market-data/live-equity-market"),
    )
    .await
}

/// Fetches SPAN margins (extracts first file from ZIP archive).
pub async fn span_margins(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
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

/// Fetches the F&O security ban list as a CSV for a specific date.
pub async fn fo_sec_ban_csv(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
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

/// Fetches participant-wise trading volumes (CSV) for a given date.
pub async fn participant_volume(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
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

/// Fetches client-wise OI limits (LST file) for a given date.
pub async fn oi_client_limits(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    // NSE expects the date in UPPERCASE DD-MON-YYYY form (e.g. "15-JAN-2024").
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
