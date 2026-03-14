use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{
    ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, HeaderMap, HeaderValue, PRAGMA, USER_AGENT,
};
use std::sync::RwLock;
use std::time::Duration;

mod common;
mod corporate;
mod derivatives;
mod equities;
mod indices;
mod macro_data;
mod models;
mod slb;
mod surveillance;

use std::sync::atomic::{AtomicUsize, Ordering};

static USER_AGENT_INDEX: AtomicUsize = AtomicUsize::new(0);

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
];

/// Helper to convert recursive serde_json::Value to PyObject using Bound types for efficiency.
fn to_py_obj(py: Python<'_>, value: serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => b.into_py_any(py),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py_any(py)
            } else {
                n.to_string().into_py_any(py)
            }
        }
        serde_json::Value::String(s) => s.into_py_any(py),
        serde_json::Value::Array(v) => {
            let list = pyo3::types::PyList::empty(py);
            for item in v {
                list.append(to_py_obj(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_json::Value::Object(m) => {
            let dict = pyo3::types::PyDict::new(py);
            for (k, v) in m {
                dict.set_item(k, to_py_obj(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

#[pyclass]
struct FinanceClient {
    client: Client,
    last_refresh: RwLock<Option<std::time::Instant>>,
}

impl FinanceClient {
    fn _refresh_session(&self) -> PyResult<()> {
        {
            let last_refresh = self
                .last_refresh
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(instant) = *last_refresh {
                if instant.elapsed() < Duration::from_secs(900) {
                    return Ok(());
                }
            }
        }

        let mut last_refresh = self
            .last_refresh
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Double-check after acquiring write lock
        if let Some(instant) = *last_refresh {
            if instant.elapsed() < Duration::from_secs(900) {
                return Ok(());
            }
        }

        let response = self
            .client
            .get(crate::common::NSE_ALL_REPORTS_URL)
            .send()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        response
            .error_for_status()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        *last_refresh = Some(std::time::Instant::now());
        Ok(())
    }

    /// Generic helper to fetch a JSON response and parse it into a PyDict.
    fn fetch_json_to_py<F>(&self, py: Python<'_>, fetch_fn: F) -> PyResult<PyObject>
    where
        F: FnOnce(&Client) -> PyResult<bytes::Bytes> + Send,
    {
        let json_bytes = py.allow_threads(|| {
            self._refresh_session()?;
            fetch_fn(&self.client)
        })?;

        let value: serde_json::Value = serde_json::from_slice(&json_bytes).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("JSON parse error: {}", e))
        })?;
        to_py_obj(py, value)
    }

    fn fetch_json_to_typed<'py, F, T>(&self, py: Python<'py>, fetch_fn: F) -> PyResult<PyObject>
    where
        F: FnOnce(&Client) -> PyResult<bytes::Bytes> + Send,
        T: for<'de> serde::Deserialize<'de> + IntoPyObject<'py>,
    {
        let json_bytes = py.allow_threads(|| {
            self._refresh_session()?;
            fetch_fn(&self.client)
        })?;
        common::parse_json_to_py_typed::<T>(py, &json_bytes)
    }
}

#[pymethods]
impl FinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let mut headers = HeaderMap::new();
        let idx = USER_AGENT_INDEX.fetch_add(1, Ordering::SeqCst) % USER_AGENTS.len();
        headers.insert(USER_AGENT, HeaderValue::from_str(USER_AGENTS[idx]).unwrap());
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));

        let client = ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .timeout(Duration::from_secs(15))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(20)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(FinanceClient {
            client,
            last_refresh: RwLock::new(None),
        })
    }

    fn _initialize_session(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| self._refresh_session())
    }

    /// Returns the current market status for all segments.
    fn get_market_status(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_typed::<_, models::MarketStatusResponse>(py, |c| {
            macro_data::market_status(c)
        })
    }

    /// Returns the market holiday calendar for the current year.
    fn get_holidays(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| macro_data::holidays(c))
    }

    /// Returns FII and DII trading activity for the most recent day.
    fn get_fii_dii_activity(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_typed::<_, Vec<models::FiiDiiActivity>>(py, |c| {
            macro_data::fii_dii_activity(c)
        })
    }

    /// Returns the summarized market turnover across segments.
    fn get_market_turnover(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| macro_data::market_turnover(c))
    }

    /// Returns historical price and volume data for a given security.
    fn price_volume_data(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_bytes =
                equities::price_volume_data(&self.client, &symbol, &from_date, &to_date)?;
            Python::with_gil(|py| {
                common::parse_csv_to_py_typed::<models::PriceVolumeRow>(py, &csv_bytes)
            })
        })
    }

    /// Returns deliverable position data for a given security.
    fn deliverable_position_data(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str =
                equities::deliverable_position_data(&self.client, &symbol, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns the Equity Bhavcopy (all records) for a given date.
    fn bhav_copy_equities(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::bhav_copy_equities(&self.client, &date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns a list of all active equities listed on NSE.
    fn get_equity_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::equity_list(&self.client)?;
            Python::with_gil(|py| common::parse_csv_to_py_typed::<models::EquityInfo>(py, &csv_str))
        })
    }

    /// Returns bulk deal data for a specific date range.
    fn bulk_deal_data(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::bulk_deal_data(&self.client, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns block deals data for a specific date range.
    fn block_deals_data(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::block_deals_data(&self.client, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns short selling data for a specific date range.
    fn short_selling_data(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::short_selling_data(&self.client, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns 52-week high or low stock records.
    fn get_52week_high_low(&self, py: Python<'_>, mode: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::fifty_two_week_high_low(c, &mode))
    }

    /// Returns the top gainers for the current market state.
    fn get_top_gainers(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::top_gainers(c))
    }

    /// Returns the top losers for the current market state.
    fn get_top_losers(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::top_losers(c))
    }

    /// Returns the most active securities for a given index.
    fn get_most_active(&self, py: Python<'_>, mode: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::most_active(c, &mode))
    }

    /// Returns the advances and declines count for all indices.
    fn get_advances_declines(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::advances_declines(c))
    }

    /// Returns monthly settlement statistics for a given financial year.
    fn get_monthly_settlement_stats(&self, py: Python<'_>, fin_year: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::monthly_settlement_stats(c, &fin_year))
    }

    /// Returns a detailed quote for a given equity symbol.
    fn get_equity_quote(&self, py: Python<'_>, symbol: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| equities::equity_quote(c, &symbol))
    }

    /// Returns a list of all NSE market indices and their current values.
    fn get_all_indices(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| indices::all_indices(c))
    }

    /// Returns constituent stocks for a specific index (e.g., 'NIFTY 50').
    fn get_index_constituents(&self, py: Python<'_>, index: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| indices::index_constituents(c, &index))
    }

    /// Returns historical OHLCV data for a specific index.
    fn get_index_history(
        &self,
        py: Python<'_>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| {
            indices::index_history(c, &index, &from_date, &to_date)
        })
    }

    /// Returns P/E, P/B and Dividend Yield for a specific index.
    fn get_index_yield(
        &self,
        py: Python<'_>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| {
            indices::index_yield(c, &index, &from_date, &to_date)
        })
    }

    /// Returns historical India VIX values.
    fn get_india_vix_history(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| indices::india_vix_history(c, &from_date, &to_date))
    }

    /// Returns Total Returns Index (TRI) historical data.
    fn get_total_returns_index(
        &self,
        py: Python<'_>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| {
            indices::total_returns_index(c, &index, &from_date, &to_date)
        })
    }

    /// Returns the option chain for a given symbol (Index or Equity).
    fn get_option_chain(
        &self,
        py: Python<'_>,
        symbol: String,
        is_index: bool,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| derivatives::option_chain(c, &symbol, is_index))
    }

    /// Returns the Derivatives (F&O) Bhavcopy for a given date and segment.
    fn bhav_copy_derivatives(
        &self,
        py: Python<'_>,
        date: String,
        segment: String,
    ) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = derivatives::bhav_copy_derivatives(&self.client, &date, &segment)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns the list of securities currently in the F&O Ban period.
    fn get_fo_sec_ban(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| derivatives::fo_sec_ban(c))
    }

    /// Returns the SPAN margins for a given date.
    fn get_span_margins(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = derivatives::span_margins(&self.client, &date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns upcoming corporate actions like dividends, splits, etc.
    fn get_corporate_actions(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| corporate::corporate_actions(c))
    }

    /// Returns financial results metadata for a specific security.
    fn get_financial_results(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
        period: String,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| {
            corporate::financial_results(c, &symbol, &from_date, &to_date, &period)
        })
    }

    /// Parses financial statement data from an XBRL URL provided by NSE.
    fn get_financial_details(&self, py: Python<'_>, xbrl_url: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| corporate::parse_xbrl_data(c, &xbrl_url))
    }

    /// Returns the SLB (Securities Lending & Borrowing) Bhavcopy for a given date.
    fn get_slb_bhavcopy(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = slb::slb_bhavcopy(&self.client, &date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    // --- Granular Equity Reports ---

    // --- Granular Macro Reports ---
    /// Returns FII/DII activity statistics (XLS format).
    fn get_fii_stats(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let bytes = macro_data::fii_stats(&self.client, &date)?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    // --- Granular Derivative Reports ---
    /// Returns FO security ban list as CSV for a given date.
    fn get_fo_ban_list(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = derivatives::fo_sec_ban_csv(&self.client, &date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns participant wise trading volumes for a given date.
    fn get_participant_volume(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = derivatives::participant_volume(&self.client, &date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns client wise OI limits (LST) for a given date.
    fn get_oi_limits_cli(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let lst_content = derivatives::oi_client_limits(&self.client, &date)?;
            // LST is fixed-width or space-separated, but common::parse_csv_to_py might handle it if we treat it as CSV with space?
            // Actually, let's just return raw text for LST or try to parse it.
            // For now, let's treat it as space-separated CSV for basic structure.
            Python::with_gil(|py| Ok(lst_content.into_py_any(py)?))
        })
    }

    // --- Surveillance & SLB Expansion ---
    /// Returns Additional Surveillance Measure (ASM) stocks list.
    fn get_asm_stocks(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| surveillance::asm_stocks(c))
    }

    /// Returns Graded Surveillance Measure (GSM) stocks list.
    fn get_gsm_stocks(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_typed::<_, Vec<models::GSMStock>>(py, |c| surveillance::gsm_stocks(c))
    }

    /// Returns only short-term ASM stocks.
    fn get_short_ban_stocks(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = py.allow_threads(|| {
            self._refresh_session()?;
            surveillance::asm_stocks(&self.client)
        })?;
        let mut value: serde_json::Value = serde_json::from_slice(&json_bytes).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("JSON parse error: {}", e))
        })?;

        if let Some(shortterm) = value.get_mut("shortterm") {
            to_py_obj(py, shortterm.take())
        } else {
            to_py_obj(py, serde_json::json!([]))
        }
    }

    /// Returns SLB eligible securities suggestions.
    fn get_slb_eligible(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| slb::slb_eligible(c))
    }

    /// Returns SLB live analysis/open positions for a specific series.
    fn get_slb_open_positions(&self, py: Python<'_>, series: String) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| slb::live_analysis_slb(c, &series))
    }

    /// Returns SLB series master (available months).
    fn get_slb_series_master(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| slb::slb_series_master(c))
    }

    /// Returns insider trades (PIT) data for a given date range.
    fn get_insider_trades(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        self.fetch_json_to_py(py, |c| corporate::insider_trades(c, &from_date, &to_date))
    }
}

#[pymodule]
fn financeindia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FinanceClient>()?;
    m.add_class::<models::FiiDiiActivity>()?;
    m.add_class::<models::MarketStatus>()?;
    m.add_class::<models::MarketStatusResponse>()?;
    m.add_class::<models::Holiday>()?;
    m.add_class::<models::ASMStock>()?;
    m.add_class::<models::GSMStock>()?;
    m.add_class::<models::EquityInfo>()?;
    m.add_class::<models::PriceVolumeRow>()?;
    Ok(())
}
