use crate::error::FinanceResult;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[pyclass]
pub struct AsyncFinanceClient {
    client: Client,
    last_refresh: Arc<RwLock<Option<Instant>>>,
}

impl AsyncFinanceClient {
    async fn _refresh_session_async(
        client: &Client,
        last_refresh: &RwLock<Option<Instant>>,
    ) -> FinanceResult<()> {
        {
            let lock = last_refresh.read().await;
            if let Some(instant) = *lock {
                if instant.elapsed() < crate::common::SESSION_REFRESH_INTERVAL {
                    return Ok(());
                }
            }
        }

        let mut lock = last_refresh.write().await;
        if let Some(instant) = *lock {
            if instant.elapsed() < crate::common::SESSION_REFRESH_INTERVAL {
                return Ok(());
            }
        }

        let response = client
            .get(crate::common::NSE_ALL_REPORTS_URL)
            .send()
            .await?;

        response.error_for_status()?;

        *lock = Some(Instant::now());
        Ok(())
    }
}

/// Centralized macro to reduce boilerplate in async methods.
/// Handles client cloning, lock cloning, session refresh, and future conversion.
macro_rules! dispatch_async {
    ($self:expr, $py:expr, $client:ident, $body:expr) => {{
        let $client = $self.client.clone();
        let refresh_lock = $self.last_refresh.clone();
        future_into_py($py, async move {
            let $client = &$client;
            Self::_refresh_session_async($client, &refresh_lock)
                .await
                .map_err(PyErr::from)?;
            $body
        })
    }};
}

#[pymethods]
impl AsyncFinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let client = crate::common::build_client(None)?;
        Ok(AsyncFinanceClient {
            client,
            last_refresh: Arc::new(RwLock::new(None)),
        })
    }

    fn _initialize_session<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, Python::with_gil(|py| Ok(().into_py_any(py)?)))
    }

    fn get_market_status<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::market_status(&client).await?;
            Python::with_gil(|py| {
                crate::common::parse_json_to_py_typed::<crate::models::MarketStatusResponse>(
                    py,
                    &bytes,
                )
            })
        })
    }

    fn get_holidays<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::holidays(&client).await?;
            Python::with_gil(|py| {
                crate::common::parse_json_to_py_typed::<Vec<crate::models::Holiday>>(py, &bytes)
            })
        })
    }

    fn get_fii_dii_activity<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::fii_dii_activity(&client).await?;
            Python::with_gil(|py| {
                crate::common::parse_json_to_py_typed::<Vec<crate::models::FiiDiiActivity>>(py, &bytes)
            })
        })
    }

    fn get_market_turnover<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::market_turnover(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn price_volume_data_raw<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes =
                crate::equities::price_volume_data(&client, &symbol, &from_date, &to_date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn price_volume_data<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str =
                crate::equities::price_volume_data(&client, &symbol, &from_date, &to_date).await?;
            Python::with_gil(|py| {
                crate::common::parse_csv_to_py_typed::<crate::models::PriceVolumeRow>(py, &csv_str)
            })
        })
    }

    fn deliverable_position_data_raw<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes =
                crate::equities::deliverable_position_data(&client, &symbol, &from_date, &to_date)
                    .await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn deliverable_position_data<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str =
                crate::equities::deliverable_position_data(&client, &symbol, &from_date, &to_date)
                    .await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn bhav_copy_equities_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::bhav_copy_equities(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn bhav_copy_equities<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::equities::bhav_copy_equities(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_equity_list_raw<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::equity_list(&client).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_equity_list<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::equities::equity_list(&client).await?;
            Python::with_gil(|py| {
                crate::common::parse_csv_to_py_typed::<crate::models::EquityInfo>(py, &csv_str)
            })
        })
    }

    fn bulk_deal_data_raw<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::bulk_deal_data(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn bulk_deal_data<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::equities::bulk_deal_data(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn block_deals_data_raw<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::block_deals_data(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn block_deals_data<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::equities::block_deals_data(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn short_selling_data_raw<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::short_selling_data(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn short_selling_data<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str =
                crate::equities::short_selling_data(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_52week_high_low<'py>(
        &self,
        py: Python<'py>,
        mode: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::fifty_two_week_high_low(&client, &mode).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_top_gainers<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::top_gainers(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_top_losers<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::top_losers(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_most_active<'py>(&self, py: Python<'py>, mode: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::most_active(&client, &mode).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_advances_declines<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::advances_declines(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_monthly_settlement_stats<'py>(
        &self,
        py: Python<'py>,
        fin_year: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::monthly_settlement_stats(&client, &fin_year).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_equity_quote<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::equities::equity_quote(&client, &symbol).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_all_indices<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::indices::all_indices(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_index_constituents<'py>(
        &self,
        py: Python<'py>,
        index: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::indices::index_constituents(&client, &index).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_index_history<'py>(
        &self,
        py: Python<'py>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes =
                crate::indices::index_history(&client, &index, &from_date, &to_date).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_index_yield<'py>(
        &self,
        py: Python<'py>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes =
                crate::indices::index_yield(&client, &index, &from_date, &to_date).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_india_vix_history<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes =
                crate::indices::india_vix_history(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_total_returns_index<'py>(
        &self,
        py: Python<'py>,
        index: String,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes =
                crate::indices::total_returns_index(&client, &index, &from_date, &to_date).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_option_chain<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        is_index: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::derivatives::option_chain(&client, &symbol, is_index).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn bhav_copy_derivatives_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
        segment: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::bhav_copy_derivatives(&client, &date, &segment).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn bhav_copy_derivatives<'py>(
        &self,
        py: Python<'py>,
        date: String,
        segment: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str =
                crate::derivatives::bhav_copy_derivatives(&client, &date, &segment).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_fo_sec_ban<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::derivatives::fo_sec_ban(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_span_margins_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::span_margins(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_span_margins<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::derivatives::span_margins(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_fo_ban_list_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::fo_sec_ban_csv(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_fo_ban_list<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::derivatives::fo_sec_ban_csv(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_participant_volume_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::participant_volume(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_participant_volume<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::derivatives::participant_volume(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_oi_limits_cli_raw<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::oi_client_limits(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_oi_limits_cli<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::oi_client_limits(&client, &date).await?;
            Python::with_gil(|py| {
                let s = String::from_utf8(bytes.to_vec())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyUnicodeDecodeError, _>(e.to_string()))?;
                Ok(s.into_py_any(py)?)
            })
        })
    }

    fn get_corporate_actions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::corporate::corporate_actions(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_financial_results<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: String,
        to_date: String,
        period: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::corporate::financial_results(
                &client, &symbol, &from_date, &to_date, &period,
            )
            .await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_financial_details<'py>(
        &self,
        py: Python<'py>,
        xbrl_url: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::corporate::parse_xbrl_data(&client, &xbrl_url).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_insider_trades<'py>(
        &self,
        py: Python<'py>,
        from_date: String,
        to_date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes =
                crate::corporate::insider_trades(&client, &from_date, &to_date).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_slb_bhavcopy_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::slb::slb_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_slb_bhavcopy<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::slb::slb_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_slb_eligible<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::slb::slb_eligible(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_slb_open_positions<'py>(
        &self,
        py: Python<'py>,
        series: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::slb::live_analysis_slb(&client, &series).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_slb_series_master<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::slb::slb_series_master(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_fii_stats<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::fii_stats(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_asm_stocks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::asm_stocks(&client).await?;
            Python::with_gil(|py| {
                crate::common::parse_json_to_py_typed::<Vec<crate::models::ASMStock>>(py, &bytes)
            })
        })
    }

    fn get_gsm_stocks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::equities::gsm_stocks(&client).await?;
            Python::with_gil(|py| {
                crate::common::parse_json_to_py_typed::<Vec<crate::models::GSMStock>>(py, &bytes)
            })
        })
    }

    fn get_short_ban_stocks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::derivatives::fo_sec_ban(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_live_currency_market<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::currency::live_currency_market(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_currency_bhavcopy_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::currency::currency_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_currency_bhavcopy<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::currency::currency_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_live_commodities_market<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let json_bytes = crate::commodities::nse_live_commodities_market(&client).await?;
            Python::with_gil(|py| {
                let value = crate::common::parse_json_value(&json_bytes)?;
                crate::to_py_obj(py, value)
            })
        })
    }

    fn get_nse_commodities_bhavcopy_raw<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::commodities::nse_commodities_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }

    fn get_nse_commodities_bhavcopy<'py>(
        &self,
        py: Python<'py>,
        date: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let csv_str = crate::commodities::nse_commodities_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| crate::common::parse_csv_to_py(py, &csv_str))
        })
    }

    fn get_mcx_bhavcopy<'py>(&self, py: Python<'py>, date: String) -> PyResult<Bound<'py, PyAny>> {
        dispatch_async!(self, py, client, {
            let bytes = crate::commodities::mcx_bhavcopy(&client, &date).await?;
            Python::with_gil(|py| Ok(pyo3::types::PyBytes::new(py, &bytes).into_any().unbind()))
        })
    }
}
