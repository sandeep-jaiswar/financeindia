use crate::common::{fetch_bytes, parse_date_robust, read_first_text_file_from_zip};
use crate::error::{FinanceError, FinanceResult};
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode, utf8_percent_encode};
use reqwest::Client;

/// Fetches the Equity Bhavcopy (UDiFF format) for a given date.
pub async fn bhav_copy_equities(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/content/cm/BhavCopy_NSE_CM_0_0_0_{}_F_0000.csv.zip",
        d.format("%Y%m%d")
    );

    let bytes = fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await?;
    read_first_text_file_from_zip(bytes)
}

/// Fetches historical price and volume data for a given security.
pub async fn price_volume_data(
    client: &Client,
    symbol: &str,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/generateSecurityWiseHistoricalData?from={}&to={}&symbol={}&type=priceVolume&series=ALL&csv=true",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT),
        encoded_symbol
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/report-detail/eq_security"),
    )
    .await
}

/// Fetches bulk deal data for a date range.
pub async fn bulk_deal_data(
    client: &Client,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=bulk_deals&from={}&to={}&csv=true",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/report-detail/display-bulk-and-block-deals"),
    )
    .await
}

/// Fetches block deals data for a date range.
pub async fn block_deals_data(
    client: &Client,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=block_deals&from={}&to={}&csv=true",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/report-detail/display-bulk-and-block-deals"),
    )
    .await
}

/// Fetches short selling data for a date range.
pub async fn short_selling_data(
    client: &Client,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/bulk-block-short-deals?optionType=short_selling&from={}&to={}&csv=true",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT)
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/report-detail/display-bulk-and-block-deals"),
    )
    .await
}

/// Fetches advances and declines data.
pub async fn advances_declines(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/equity-stockIndices?index=ALL%20INDICES";
    fetch_bytes(client, url, Some("https://www.nseindia.com/market-data/live-equity-market")).await
}

/// Fetches monthly settlement statistics.
pub async fn monthly_settlement_stats(client: &Client, fin_year: &str) -> FinanceResult<Bytes> {
    // fin_year format: YYYY-YYYY
    let parts: Vec<&str> = fin_year.split('-').collect();
    if parts.len() != 2 || parts[0].len() != 4 || parts[1].len() != 4 {
        return Err(FinanceError::Runtime(
            "Invalid fin_year format. Expected YYYY-YYYY (e.g., 2024-2025).".to_string(),
        ));
    }

    let y1: u32 = parts[0]
        .parse()
        .map_err(|_| FinanceError::Runtime("Invalid start year".to_string()))?;
    let y2: u32 = parts[1]
        .parse()
        .map_err(|_| FinanceError::Runtime("Invalid end year".to_string()))?;

    if y2 != y1 + 1 {
        return Err(FinanceError::Runtime(
            "Invalid financial year. End year must be Start year + 1.".to_string(),
        ));
    }

    let url = format!(
        "https://www.nseindia.com/api/historicalOR/monthly-sett-stats-data?finYear={}",
        fin_year
    );
    fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches 52 week high/low data.
pub async fn fifty_two_week_high_low(client: &Client, mode: &str) -> FinanceResult<Bytes> {
    let url = if mode == "low" {
        "https://www.nseindia.com/api/live-analysis-data-52weeklowstock"
    } else {
        "https://www.nseindia.com/api/live-analysis-data-52weekhighstock"
    };
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches most active securities.
pub async fn most_active(client: &Client, mode: &str) -> FinanceResult<Bytes> {
    let encoded_mode = utf8_percent_encode(mode, NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/live-analysis-most-active-securities?index={}",
        encoded_mode
    );
    fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches top gainers.
pub async fn top_gainers(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/live-analysis-variations?index=gainers";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches top losers.
pub async fn top_losers(client: &Client) -> FinanceResult<Bytes> {
    // NOTE: NSE's API endpoint intentionally uses "loosers" (their typo). Do not "correct" this.
    let url = "https://www.nseindia.com/api/live-analysis-variations?index=loosers";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches deliverable position data for a given security.
pub async fn deliverable_position_data(
    client: &Client,
    symbol: &str,
    from_date: &str,
    to_date: &str,
) -> FinanceResult<Bytes> {
    let from = parse_date_robust(from_date)?;
    let to = parse_date_robust(to_date)?;
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/historicalOR/generateSecurityWiseHistoricalData?from={}&to={}&symbol={}&type=deliverable&series=ALL&csv=true",
        from.format(crate::common::NSE_DATE_FMT),
        to.format(crate::common::NSE_DATE_FMT),
        encoded_symbol
    );
    fetch_bytes(
        client,
        &url,
        Some("https://www.nseindia.com/report-detail/eq_security"),
    )
    .await
}

/// Fetches the list of all active equities.
pub async fn equity_list(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://archives.nseindia.com/content/equities/EQUITY_L.csv";
    fetch_bytes(client, url, Some("https://www.nseindia.com")).await
}

/// Fetches a detailed quote for an equity symbol.
pub async fn equity_quote(client: &Client, symbol: &str) -> FinanceResult<Bytes> {
    let encoded_symbol = percent_encode(symbol.as_bytes(), NON_ALPHANUMERIC).to_string();
    let url = format!(
        "https://www.nseindia.com/api/quote-equity?symbol={}",
        encoded_symbol
    );
    fetch_bytes(
        client,
        &url,
        Some(&format!(
            "https://www.nseindia.com/get-quotes/equity?symbol={}",
            encoded_symbol
        )),
    )
    .await
}

/// Fetches Additional Surveillance Measure (ASM) stocks.
pub async fn asm_stocks(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/reportASM";
    fetch_bytes(client, url, Some("https://www.nseindia.com/reports/asm")).await
}

/// Fetches Graded Surveillance Measure (GSM) stocks.
pub async fn gsm_stocks(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/reportGSM";
    fetch_bytes(client, url, Some("https://www.nseindia.com/reports/gsm")).await
}

/// Fetches FII/DII trading activity.
pub async fn fii_dii_activity(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/fiidiiTradeReact";
    fetch_bytes(client, url, Some("https://www.nseindia.com/reports/fii-dii")).await
}

/// Fetches detailed FII statistics (.xls).
pub async fn fii_stats(client: &Client, date: &str) -> FinanceResult<Bytes> {
    let d = parse_date_robust(date)?;
    let url = format!(
        "https://nsearchives.nseindia.com/content/fo/fii_stats_{}.xls",
        d.format("%d-%b-%Y")
    );
    fetch_bytes(client, &url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches market turnover.
pub async fn market_turnover(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/market-turnover-popup";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Fetches market holidays.
pub async fn holidays(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/holiday-master?type=trading";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}

/// Returns the current market status.
pub async fn market_status(client: &Client) -> FinanceResult<Bytes> {
    let url = "https://www.nseindia.com/api/marketStatus";
    fetch_bytes(client, url, Some(crate::common::NSE_ALL_REPORTS_URL)).await
}
