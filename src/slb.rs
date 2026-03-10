use pyo3::prelude::*;
use reqwest::blocking::Client;
use crate::common::{parse_date_robust, fetch_text};

/// Fetches SLB Bhavcopy (DAT).
pub fn slb_bhavcopy(client: &Client, date: &str) -> PyResult<String> {
    let d = parse_date_robust(date)?;
    // Pattern: https://nsearchives.nseindia.com/archives/slbs/bhavcopy/SLBM_BC_09032026.DAT
    let url = format!(
        "https://nsearchives.nseindia.com/archives/slbs/bhavcopy/SLBM_BC_{}.DAT",
        d.format("%d%m%Y")
    );
    fetch_text(client, &url, Some("https://www.nseindia.com/market-data/securities-lending-and-borrowing"))
}

