use pyo3::prelude::*;
use serde::Deserialize;

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct FiiDiiActivity {
    #[serde(rename = "buyValue")]
    pub buy_value: Option<String>,
    pub category: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "netValue")]
    pub net_value: Option<String>,
    #[serde(rename = "sellValue")]
    pub sell_value: Option<String>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct MarketStatus {
    pub market: Option<String>,
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
    #[serde(rename = "sr_no")]
    pub sr_no: i32,
    pub description: Option<String>,
    #[serde(rename = "tradingDate")]
    pub trading_date: Option<String>,
    pub week_day: Option<String>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct ASMStock {
    pub indicator: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    pub symbol: Option<String>,
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
#[derive(Debug, Clone, Deserialize)]
pub struct EquityInfo {
    pub symbol: Option<String>,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    pub series: Option<String>,
    #[serde(rename = "listingDate")]
    pub listing_date: Option<String>,
    #[serde(rename = "paidUpValue")]
    pub paid_up_value: Option<f64>,
    #[serde(rename = "marketLot")]
    pub market_lot: Option<String>,
    pub isin: Option<String>,
    #[serde(rename = "faceValue")]
    pub face_value: Option<f64>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct PriceVolumeRow {
    #[serde(rename = "SYMBOL")]
    pub symbol: Option<String>,
    #[serde(rename = "SERIES")]
    pub series: Option<String>,
    #[serde(rename = "DATE1")]
    pub date: Option<String>,
    #[serde(rename = "PREV_CLOSE")]
    pub prev_close: Option<String>,
    #[serde(rename = "OPEN_PRICE")]
    pub open_price: Option<String>,
    #[serde(rename = "HIGH_PRICE")]
    pub high_price: Option<String>,
    #[serde(rename = "LOW_PRICE")]
    pub low_price: Option<String>,
    #[serde(rename = "LAST_PRICE")]
    pub last_price: Option<String>,
    #[serde(rename = "CLOSE_PRICE")]
    pub close_price: Option<String>,
    #[serde(rename = "AVG_PRICE")]
    pub average_price: Option<String>,
    #[serde(rename = "TTL_TRD_QNTY")]
    pub total_traded_quantity: Option<String>,
    #[serde(rename = "TURNOVER_LACS")]
    pub turnover: Option<String>,
    #[serde(rename = "NO_OF_TRADES")]
    pub no_of_trades: Option<String>,
}
