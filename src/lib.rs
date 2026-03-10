use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::IntoPyObjectExt;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, PRAGMA};
use std::time::Duration;
use std::sync::RwLock;

mod common;
mod equities;
mod indices;
mod derivatives;
mod slb;
mod macro_data;
mod corporate;

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
            Ok(list.to_object(py))
        }
        serde_json::Value::Object(m) => {
            let dict = pyo3::types::PyDict::new(py);
            for (k, v) in m {
                dict.set_item(k, to_py_obj(py, v)?)?;
            }
            Ok(dict.to_object(py))
        }
    }
}

#[pyclass]
struct FinanceClient {
    client: Client,
    last_refresh: RwLock<Option<std::time::Instant>>,
}

#[pymethods]
impl FinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"));
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
            last_refresh: RwLock::new(None) 
        })
    }

    fn _initialize_session(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| self._refresh_session())
    }

    fn _refresh_session(&self) -> PyResult<()> {
        {
            let last_refresh = self.last_refresh.read().unwrap();
            if let Some(instant) = *last_refresh {
                if instant.elapsed() < Duration::from_secs(900) {
                    return Ok(());
                }
            }
        }
        
        let mut last_refresh = self.last_refresh.write().unwrap();
        // Double-check after acquiring write lock
        if let Some(instant) = *last_refresh {
            if instant.elapsed() < Duration::from_secs(900) {
                return Ok(());
            }
        }

        let response = self.client.get("https://www.nseindia.com/all-reports")
            .send()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        response.error_for_status()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        *last_refresh = Some(std::time::Instant::now());
        Ok(())
    }

    /// Returns the current market status for all segments.
    fn get_market_status(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = macro_data::market_status(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the market holiday calendar for the current year.
    fn get_holidays(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = macro_data::holidays(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns FII and DII trading activity for the most recent day.
    fn get_fii_dii_activity(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = macro_data::fii_dii_activity(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the summarized market turnover across segments.
    fn get_market_turnover(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = macro_data::market_turnover(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns historical price and volume data for a given security.
    fn price_volume_data(&self, py: Python<'_>, symbol: String, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::price_volume_data(&self.client, &symbol, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns deliverable position data for a given security.
    fn deliverable_position_data(&self, py: Python<'_>, symbol: String, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::deliverable_position_data(&self.client, &symbol, &from_date, &to_date)?;
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
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns bulk deal data for a specific date range.
    fn bulk_deal_data(&self, py: Python<'_>, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::bulk_deal_data(&self.client, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns block deals data for a specific date range.
    fn block_deals_data(&self, py: Python<'_>, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::block_deals_data(&self.client, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns short selling data for a specific date range.
    fn short_selling_data(&self, py: Python<'_>, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = equities::short_selling_data(&self.client, &from_date, &to_date)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns 52-week high or low stock records.
    fn get_52week_high_low(&self, py: Python<'_>, mode: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = equities::fifty_two_week_high_low(&self.client, &mode)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the top gainers for the current market state.
    fn get_top_gainers(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = equities::top_gainers(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the top losers for the current market state.
    fn get_top_losers(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = equities::top_losers(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the most active securities for a given index.
    fn get_most_active(&self, py: Python<'_>, mode: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = equities::most_active(&self.client, &mode)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the advances and declines count for all indices.
    fn get_advances_declines(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = equities::advances_declines(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    fn get_monthly_settlement_stats(&self, py: Python<'_>, fin_year: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            equities::monthly_settlement_stats(&self.client, &fin_year)
        })
    }

    fn get_equity_quote(&self, py: Python<'_>, symbol: String) -> PyResult<String> {
        py.allow_threads(|| {
            self._refresh_session()?;
            equities::equity_quote(&self.client, &symbol)
        })
    }

    /// Returns a list of all NSE market indices and their current values.
    fn get_all_indices(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = indices::all_indices(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns constituent stocks for a specific index (e.g., 'NIFTY 50').
    fn get_index_constituents(&self, py: Python<'_>, index: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = indices::index_constituents(&self.client, &index)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns historical OHLCV data for a specific index.
    fn get_index_history(&self, py: Python<'_>, index: String, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = indices::index_history(&self.client, &index, &from_date, &to_date)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns P/E, P/B and Dividend Yield for a specific index.
    fn get_index_yield(&self, py: Python<'_>, index: String, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = indices::index_yield(&self.client, &index, &from_date, &to_date)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns historical India VIX values.
    fn get_india_vix_history(&self, py: Python<'_>, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = indices::india_vix_history(&self.client, &from_date, &to_date)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns Total Returns Index (TRI) historical data.
    fn get_total_returns_index(&self, py: Python<'_>, index: String, from_date: String, to_date: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = indices::total_returns_index(&self.client, &index, &from_date, &to_date)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the option chain for a given symbol (Index or Equity).
    fn get_option_chain(&self, py: Python<'_>, symbol: String, is_index: bool) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = derivatives::option_chain(&self.client, &symbol, is_index)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns the Derivatives (F&O) Bhavcopy for a given date and segment.
    fn bhav_copy_derivatives(&self, py: Python<'_>, date: String, segment: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let csv_str = derivatives::bhav_copy_derivatives(&self.client, &date, &segment)?;
            Python::with_gil(|py| common::parse_csv_to_py(py, &csv_str))
        })
    }

    /// Returns the list of securities currently in the F&O Ban period.
    fn get_fo_sec_ban(&self, py: Python<'_>) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = derivatives::fo_sec_ban(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
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
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = corporate::corporate_actions(&self.client)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Returns financial results metadata for a specific security.
    fn get_financial_results(&self, py: Python<'_>, symbol: String, from_date: String, to_date: String, period: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = corporate::financial_results(&self.client, &symbol, &from_date, &to_date, &period)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
    }

    /// Parses financial statement data from an XBRL URL provided by NSE.
    fn get_financial_details(&self, py: Python<'_>, xbrl_url: String) -> PyResult<PyObject> {
        py.allow_threads(|| {
            self._refresh_session()?;
            let json_str = corporate::parse_xbrl_data(&self.client, &xbrl_url)?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
            Python::with_gil(|py| to_py_obj(py, value))
        })
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
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).to_object(py)))
        })
    }

    // --- Granular Derivative Reports ---
}

#[pymodule]
fn financeindia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FinanceClient>()?;
    Ok(())
}