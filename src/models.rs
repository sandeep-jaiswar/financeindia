use crate::common::deserialize_optional_f64;
use pyo3::prelude::*;
use serde::Deserialize;

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct FiiDiiActivity {
    #[serde(rename = "buyValue", deserialize_with = "deserialize_optional_f64")]
    pub buy_value: Option<f64>,
    pub category: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "netValue", deserialize_with = "deserialize_optional_f64")]
    pub net_value: Option<f64>,
    #[serde(rename = "sellValue", deserialize_with = "deserialize_optional_f64")]
    pub sell_value: Option<f64>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct MarketStatus {
    pub market: Option<String>,
    #[serde(rename = "marketStatus")]
    pub status: Option<String>,
    #[serde(rename = "lastUpdateTime")]
    pub last_update_time: Option<String>,
    pub index: Option<String>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct MarketStatusResponse {
    #[serde(rename = "marketState")]
    pub market_state: Vec<MarketStatus>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct Holiday {
    /// Serial number; `None` if the API omits it.
    #[serde(rename = "sr_no")]
    pub sr_no: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "tradingDate")]
    pub trading_date: Option<String>,
    #[serde(rename = "weekDay")]
    pub week_day: Option<String>,
}

/// Wrapper for holidays API response (NSE returns {"CBM": [...]})
#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct HolidaysResponse {
    #[serde(rename = "CBM")]
    pub cbm: Vec<Holiday>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct ASMStock {
    pub symbol: Option<String>,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    #[serde(rename = "asmSurvIndicator")]
    pub indicator: Option<String>,
    #[serde(rename = "asmTime")]
    pub time: Option<String>,
}

/// Wrapper for ASM API response (NSE returns {"longterm": {"data": [...]}})
#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct ASMResponse {
    pub longterm: ASMDataWrapper,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct ASMDataWrapper {
    pub data: Vec<ASMStock>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct GSMStock {
    pub company: Option<String>,
    pub isin: Option<String>,
    pub symbol: Option<String>,
    pub stage: Option<i32>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct EquityInfo {
    pub symbol: Option<String>,
    pub company_name: Option<String>,
    pub series: Option<String>,
    pub listing_date: Option<String>,
    pub paid_up_value: Option<f64>,
    pub market_lot: Option<String>,
    pub isin: Option<String>,
    pub face_value: Option<f64>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct PriceVolumeRow {
    pub symbol: Option<String>,
    pub series: Option<String>,
    pub date: Option<String>,
    pub prev_close: Option<f64>,
    pub open_price: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub last_price: Option<f64>,
    pub close_price: Option<f64>,
    pub average_price: Option<f64>,
    /// NSE returns trade quantities as integers; kept as i64 to avoid
    /// float precision loss above 2^53.
    pub total_traded_quantity: Option<i64>,
    pub turnover: Option<f64>,
    /// Trade counts are integers (NSE may emit a `.0` float suffix).
    pub no_of_trades: Option<i64>,
}
