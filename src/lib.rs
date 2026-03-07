use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, REFERER, CACHE_CONTROL, PRAGMA};
use std::time::Duration;
mod capitalmarket;

#[pyclass]
struct FinanceClient {
    client: Client,
    last_refresh: std::sync::Mutex<Option<std::time::Instant>>,
}

#[pymethods]
impl FinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let mut headers = HeaderMap::new();
        
        // Exact headers from your working example
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));

        let client = ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(FinanceClient { 
            client, 
            last_refresh: std::sync::Mutex::new(None) 
        })
    }

    /// Initializes the background session with NSE. 
    /// This is recommended to be called once before performing other operations.
    fn _initialize_session(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| self._refresh_session())
    }

    /// Refreshes the session if it's older than 15 minutes.
    /// Internal helper used to ensure cookies are valid before NIA calls.
    fn _refresh_session(&self) -> PyResult<()> {
        let mut last_refresh = self.last_refresh.lock().unwrap();
        
        if let Some(instant) = *last_refresh {
            if instant.elapsed() < Duration::from_secs(900) {
                return Ok(());
            }
        }

        // Must hit the home page first to "bake" the cookies in the Jar
        let response = self.client.get("https://www.nseindia.com/all-reports")
            .send()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        
        response.error_for_status()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        
        *last_refresh = Some(std::time::Instant::now());
        Ok(())
    }

    /// Returns the current market status (Open/Closed) for various NSE segments.
    fn get_market_status(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let response = self.client.get("https://www.nseindia.com/api/marketStatus")
                .header(REFERER, "https://www.nseindia.com/all-reports")
                .send()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

            let checked_response = response.error_for_status()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

            checked_response.text()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Fetches historical price and volume data for a given security.
    /// symbol: Stock symbol (e.g., 'RELIANCE')
    /// from_date: Start date (format: DD-MM-YYYY)
    /// to_date: End date (format: DD-MM-YYYY)
    fn price_volume_data(&self, py: Python<'_>, symbol: String, from_date: String, to_date: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::price_volume_data(&self.client, &symbol, &from_date, &to_date)
        })
    }

    /// Fetches deliverable position data for a given security.
    fn deliverable_position_data(&self, py: Python<'_>, symbol: String, from_date: String, to_date: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::deliverable_position_data(&self.client, &symbol, &from_date, &to_date)
        })
    }

    /// Fetches the Equity Bhavcopy (UDiFF format) for a given date.
    fn bhav_copy_equities(&self, py: Python<'_>, date: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::bhav_copy_equities(&self.client, &date)
        })
    }

    /// Fetches the list of all active equities listed on NSE.
    fn equity_list(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::equity_list(&self.client)
        })
    }

    /// Fetches bulk deal data for a specific date range.
    fn bulk_deal_data(&self, py: Python<'_>, from_date: String, to_date: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::bulk_deal_data(&self.client, &from_date, &to_date)
        })
    }

    /// Fetches block deals data for a specific date range.
    fn block_deals_data(&self, py: Python<'_>, from_date: String, to_date: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::block_deals_data(&self.client, &from_date, &to_date)
        })
    }

    /// Fetches the list of Nifty 50 constituent stocks.
    fn nifty50_equity_list(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::nifty50_equity_list(&self.client)
        })
    }

    /// Fetches a list of all NSE market indices.
    fn get_all_indices(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::all_indices(&self.client)
        })
    }

    /// Fetches constituent stocks for a given index (e.g., 'NIFTY 50').
    fn get_index_constituents(&self, py: Python<'_>, index: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::index_constituents(&self.client, &index)
        })
    }

    /// Fetches top gainers for the current trading day.
    fn get_top_gainers(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::top_gainers(&self.client)
        })
    }

    /// Fetches top losers for the current trading day.
    fn get_top_losers(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::top_losers(&self.client)
        })
    }

    /// Fetches most active securities by 'volume' or 'value'.
    fn get_most_active(&self, py: Python<'_>, mode: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::most_active(&self.client, &mode)
        })
    }

    /// Fetches real-time equity quote for a given symbol.
    fn get_equity_quote(&self, py: Python<'_>, symbol: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::equity_quote(&self.client, &symbol)
        })
    }

    /// Fetches option chain data for a symbol or index.
    fn get_option_chain(&self, py: Python<'_>, symbol: String, is_index: bool) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::option_chain(&self.client, &symbol, is_index)
        })
    }

    /// Fetches market holidays for the current year.
    fn get_holidays(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::holidays(&self.client)
        })
    }

    /// Fetches upcoming corporate actions (Dividends, Splits, etc.).
    fn get_corporate_actions(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::corporate_actions(&self.client)
        })
    }

    /// Fetches financial results metadata for a given symbol and period.
    /// period: 'Quarterly', 'Annual', 'Half Yearly', etc.
    fn get_financial_results(&self, py: Python<'_>, symbol: String, from_date: String, to_date: String, period: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::financial_results(&self.client, &symbol, &from_date, &to_date, &period)
        })
    }

    /// Downloads and parses an XBRL file from a given URL into a JSON string.
    /// Use this on the 'xbrl' field from get_financial_results.
    fn get_financial_details(&self, py: Python<'_>, xbrl_url: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            capitalmarket::parse_xbrl_data(&self.client, &xbrl_url)
        })
    }
}

#[pymodule]
fn financeindia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FinanceClient>()?;
    Ok(())
}