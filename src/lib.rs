//! # FinanceIndia
//!
//! A high-performance, lightweight Python library written in Rust for fetching Indian financial market data (NSE).
//!
//! This library provides both synchronous and asynchronous clients for accessing various NSE APIs,
//! including equities, derivatives, indices, and corporate actions.

use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use reqwest::Client;
use std::sync::{RwLock, OnceLock};

mod archive;
mod async_client;
mod commodities;
mod common;
mod corporate;
mod currency;
mod derivatives;
mod error;
mod equities;
mod indices;
mod models;
mod slb;
mod streaming;

use crate::error::FinanceResult;

static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime")
    })
}

macro_rules! fetch_py {
    ($self:expr, $py:expr, $func:path $(, $arg:expr)*) => {{
        let client = $self.client.clone();
        $py.allow_threads(|| {
            runtime().block_on(async move {
                $self._refresh_session_async().await?;
                $func(&client $(, $arg)*).await
            })
        })
    }};
}

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
    async fn _refresh_session_async(&self) -> FinanceResult<()> {
        // Fast path: check under read lock to avoid exclusive acquisition on every call.
        {
            let last_refresh = self.last_refresh.read().unwrap_or_else(|p| {
                log::error!("Session read-lock was poisoned; recovering.");
                p.into_inner()
            });
            if let Some(instant) = *last_refresh {
                if instant.elapsed() < crate::common::SESSION_REFRESH_INTERVAL {
                    return Ok(());
                }
            }
        }

        // Slow path: acquire write lock and double-check (another task may have refreshed).
        let mut last_refresh = self.last_refresh.write().unwrap_or_else(|p| {
            log::error!("Session write-lock was poisoned; recovering.");
            p.into_inner()
        });
        if let Some(instant) = *last_refresh {
            if instant.elapsed() < crate::common::SESSION_REFRESH_INTERVAL {
                return Ok(());
            }
        }

        let response = self
            .client
            .get(crate::common::NSE_ALL_REPORTS_URL)
            .send()
            .await?;
        response.error_for_status()?;
        *last_refresh = Some(std::time::Instant::now());
        Ok(())
    }
}

#[pymethods]
impl FinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let client = crate::common::build_client(None)?;
        Ok(FinanceClient {
            client,
            last_refresh: RwLock::new(None),
        })
    }
    fn _initialize_session(&self, py: Python<'_>) -> PyResult<()> {
        Ok(py.allow_threads(|| crate::runtime().block_on(async { self._refresh_session_async().await }))?)
    }

    /// Returns the current market status for all segments.
    fn get_market_status(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::market_status)?;
        Ok(common::parse_json_to_py_typed::<models::MarketStatusResponse>(py, &json_bytes)?)
    }

    /// Returns the current list of NSE stock market holidays.
    fn get_holidays(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::holidays)?;
        Ok(common::parse_json_to_py_typed::<Vec<models::Holiday>>(py, &json_bytes)?)
    }

    /// Returns FII and DII trading activity for the current day.
    fn get_fii_dii_activity(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::fii_dii_activity)?;
        Ok(common::parse_json_to_py_typed::<Vec<models::FiiDiiActivity>>(py, &json_bytes)?)
    }

    /// Returns the current total market turnover across all segments.
    fn get_market_turnover(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::market_turnover)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns historical price and volume data for a given security.
    fn price_volume_data(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let csv_bytes = fetch_py!(
            self,
            py,
            equities::price_volume_data,
            &symbol,
            &from_date,
            &to_date
        )?;
        common::parse_csv_to_py_typed::<models::PriceVolumeRow>(py, &csv_bytes)
    }

    /// Returns deliverable position data for a given security.
    fn deliverable_position_data(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let csv_str = fetch_py!(
            self,
            py,
            equities::deliverable_position_data,
            &symbol,
            &from_date,
            &to_date
        )?;
        common::parse_csv_to_py(py, &csv_str)
    }

    /// Returns the Equity Bhavcopy (all records) for a given date.
    fn bhav_copy_equities(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, equities::bhav_copy_equities, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns a list of all active equities listed on NSE.
    fn get_equity_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, equities::equity_list)?;
        Ok(common::parse_csv_to_py_typed::<models::EquityInfo>(py, &csv_str)?)
    }

    /// Returns bulk deal data for a specific date range.
    fn bulk_deal_data(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, equities::bulk_deal_data, &from_date, &to_date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns block deals data for a specific date range.
    fn block_deals_data(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, equities::block_deals_data, &from_date, &to_date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns short selling data for a specific date range.
    fn short_selling_data(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, equities::short_selling_data, &from_date, &to_date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns 52-week high or low stock records.
    fn get_52week_high_low(&self, py: Python<'_>, mode: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::fifty_two_week_high_low, &mode)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the top gainers for the current market state.
    fn get_top_gainers(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::top_gainers)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the top losers for the current market state.
    fn get_top_losers(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::top_losers)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the most active securities for a given index.
    fn get_most_active(&self, py: Python<'_>, mode: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::most_active, &mode)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns advances and declines counts for all indices.
    fn get_advances_declines(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::advances_declines)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns monthly settlement statistics for a given financial year (format `"YYYY-YYYY"`).
    fn get_monthly_settlement_stats(&self, py: Python<'_>, fin_year: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::monthly_settlement_stats, &fin_year)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns a detailed quote for a given equity symbol.
    /// Returns a full JSON quote for a given equity symbol.
    fn get_equity_quote(&self, py: Python<'_>, symbol: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::equity_quote, &symbol)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns a list of all NSE market indices and their current values.
    fn get_all_indices(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, indices::all_indices)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns constituent stocks for a specific index (e.g., 'NIFTY 50').
    fn get_index_constituents(&self, py: Python<'_>, index: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, indices::index_constituents, &index)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns historical OHLCV data for a specific index.
    fn get_index_history(
        &self,
        py: Python<'_>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(
            self,
            py,
            indices::index_history,
            &index,
            &from_date,
            &to_date
        )?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns P/E, P/B and Dividend Yield for a specific index.
    fn get_index_yield(
        &self,
        py: Python<'_>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, indices::index_yield, &index, &from_date, &to_date)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns historical India VIX values.
    fn get_india_vix_history(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, indices::india_vix_history, &from_date, &to_date)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns Total Returns Index (TRI) historical data.
    fn get_total_returns_index(
        &self,
        py: Python<'_>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(
            self,
            py,
            indices::total_returns_index,
            &index,
            &from_date,
            &to_date
        )?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the option chain for a given symbol (Index or Equity).
    fn get_option_chain(
        &self,
        py: Python<'_>,
        symbol: String,
        is_index: bool,
    ) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, derivatives::option_chain, &symbol, is_index)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the Derivatives (F&O) Bhavcopy for a given date and segment.
    fn bhav_copy_derivatives(
        &self,
        py: Python<'_>,
        date: String,
        segment: String,
    ) -> PyResult<PyObject> {
        let csv_str = fetch_py!(
            self,
            py,
            derivatives::bhav_copy_derivatives,
            &date,
            &segment
        )?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns the list of securities currently in the F&O Ban period.
    fn get_fo_sec_ban(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, derivatives::fo_sec_ban)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the SPAN margins for a given date.
    fn get_span_margins(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, derivatives::span_margins, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns upcoming corporate actions like dividends, splits, etc.
    fn get_corporate_actions(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, corporate::corporate_actions)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
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
        let json_bytes = fetch_py!(
            self,
            py,
            corporate::financial_results,
            &symbol,
            &from_date,
            &to_date,
            &period
        )?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Parses financial statement data from an XBRL URL provided by NSE.
    fn get_financial_details(&self, py: Python<'_>, xbrl_url: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, corporate::parse_xbrl_data, &xbrl_url)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns the SLB (Securities Lending & Borrowing) Bhavcopy for a given date.
    fn get_slb_bhavcopy(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, slb::slb_bhavcopy, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns detailed FII statistics as raw XLS bytes for a given date.
    fn get_fii_stats(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, equities::fii_stats, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    /// Returns the F&O security ban list as CSV for a given date.
    fn get_fo_ban_list(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, derivatives::fo_sec_ban_csv, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns participant wise trading volumes for a given date.
    fn get_participant_volume(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, derivatives::participant_volume, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns client wise OI limits (LST) for a given date.
    fn get_oi_limits_cli(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let lst_content = fetch_py!(self, py, derivatives::oi_client_limits, &date)?;
        Ok(lst_content.into_py_any(py)?)
    }

    /// Returns Additional Surveillance Measure (ASM) stocks.
    fn get_asm_stocks(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::asm_stocks)?;
        Ok(common::parse_json_to_py_typed::<Vec<models::ASMStock>>(py, &json_bytes)?)
    }

    /// Returns Graded Surveillance Measure (GSM) stocks.
    fn get_gsm_stocks(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, equities::gsm_stocks)?;
        Ok(common::parse_json_to_py_typed::<Vec<models::GSMStock>>(py, &json_bytes)?)
    }

    /// Returns the list of securities currently in the F&O ban period.
    /// Note: uses the same live API as `get_fo_sec_ban`.
    fn get_short_ban_stocks(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, derivatives::fo_sec_ban)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns SLB eligible securities suggestions.
    fn get_slb_eligible(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, slb::slb_eligible)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns SLB live analysis/open positions for a specific series.
    fn get_slb_open_positions(&self, py: Python<'_>, series: String) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, slb::live_analysis_slb, &series)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns SLB series master (available months).
    fn get_slb_series_master(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, slb::slb_series_master)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns insider trades (PIT) data for a given date range.
    fn get_insider_trades(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, corporate::insider_trades, &from_date, &to_date)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }
    fn bhav_copy_equities_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, equities::bhav_copy_equities, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_equity_list_raw(&self, py: Python<'_>) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, equities::equity_list)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn bulk_deal_data_raw(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, equities::bulk_deal_data, &from_date, &to_date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn block_deals_data_raw(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, equities::block_deals_data, &from_date, &to_date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn short_selling_data_raw(
        &self,
        py: Python<'_>,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, equities::short_selling_data, &from_date, &to_date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn bhav_copy_derivatives_raw(
        &self,
        py: Python<'_>,
        date: String,
        segment: String,
    ) -> PyResult<PyObject> {
        let bytes = fetch_py!(
            self,
            py,
            derivatives::bhav_copy_derivatives,
            &date,
            &segment
        )?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_span_margins_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, derivatives::span_margins, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_fo_ban_list_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, derivatives::fo_sec_ban_csv, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_participant_volume_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, derivatives::participant_volume, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_slb_bhavcopy_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, slb::slb_bhavcopy, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_currency_bhavcopy_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, currency::currency_bhavcopy, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_nse_commodities_bhavcopy_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, commodities::nse_commodities_bhavcopy, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn price_volume_data_raw(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let bytes = fetch_py!(
            self,
            py,
            equities::price_volume_data,
            &symbol,
            &from_date,
            &to_date
        )?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn deliverable_position_data_raw(
        &self,
        py: Python<'_>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<PyObject> {
        let bytes = fetch_py!(
            self,
            py,
            equities::deliverable_position_data,
            &symbol,
            &from_date,
            &to_date
        )?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    fn get_oi_limits_cli_raw(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, derivatives::oi_client_limits, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }

    /// Returns the live Currency market status.
    fn get_live_currency_market(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, currency::live_currency_market)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns Currency Bhavcopy
    fn get_currency_bhavcopy(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, currency::currency_bhavcopy, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns the live NSE Commodities market status.
    fn get_live_commodities_market(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_bytes = fetch_py!(self, py, commodities::nse_live_commodities_market)?;
        let value = common::parse_json_value(&json_bytes)?;
        Ok(to_py_obj(py, value)?)
    }

    /// Returns NSE Commodities Bhavcopy
    fn get_nse_commodities_bhavcopy(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let csv_str = fetch_py!(self, py, commodities::nse_commodities_bhavcopy, &date)?;
        Ok(common::parse_csv_to_py(py, &csv_str)?)
    }

    /// Returns MCX Bhavcopy (ZIP/CSV bytes)
    fn get_mcx_bhavcopy(&self, py: Python<'_>, date: String) -> PyResult<PyObject> {
        let bytes = fetch_py!(self, py, commodities::mcx_bhavcopy, &date)?;
        Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind())
    }
}

#[pymodule]
fn financeindia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FinanceClient>()?;
    m.add_class::<async_client::AsyncFinanceClient>()?;
    m.add_class::<models::FiiDiiActivity>()?;
    m.add_class::<models::MarketStatus>()?;
    m.add_class::<models::MarketStatusResponse>()?;
    m.add_class::<models::Holiday>()?;
    m.add_class::<models::ASMStock>()?;
    m.add_class::<models::GSMStock>()?;
    m.add_class::<models::EquityInfo>()?;
    m.add_class::<models::PriceVolumeRow>()?;
    m.add_class::<streaming::MarketStream>()?;
    m.add_class::<archive::BhavArchive>()?;
    Ok(())
}
