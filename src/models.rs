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
    #[serde(rename = "Sr_no")]
    pub sr_no: i32,
    pub description: Option<String>,
    #[serde(rename = "tradingDate")]
    pub trading_date: Option<String>,
    pub week_day: Option<String>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct ASMStock {
    #[serde(rename = "asmSurvIndicator")]
    pub indicator: Option<String>,
    #[serde(rename = "asmTime")]
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
    #[serde(rename = "SYMBOL")]
    pub symbol: Option<String>,
    #[serde(rename = "NAME OF COMPANY")]
    pub company_name: Option<String>,
    #[serde(rename = " SERIES")]
    pub series: Option<String>,
    #[serde(rename = " DATE OF LISTING")]
    pub listing_date: Option<String>,
    #[serde(rename = " PAID UP VALUE")]
    pub paid_up_value: Option<f64>,
    #[serde(rename = " MARKET LOT")]
    pub market_lot: Option<String>,
    #[serde(rename = " ISIN NUMBER")]
    pub isin: Option<String>,
    #[serde(rename = " FACE VALUE")]
    pub face_value: Option<f64>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PriceVolumeRow {
    #[serde(rename = "Symbol  ", default)]
    pub symbol: Option<String>,
    #[serde(rename = "Series  ", default)]
    pub series: Option<String>,
    #[serde(rename = "Date  ", default)]
    pub date: Option<String>,
    #[serde(rename = "Prev Close  ", default)]
    pub prev_close: Option<String>,
    #[serde(rename = "Open Price  ", default)]
    pub open_price: Option<String>,
    #[serde(rename = "High Price  ", default)]
    pub high_price: Option<String>,
    #[serde(rename = "Low Price  ", default)]
    pub low_price: Option<String>,
    #[serde(rename = "Last Price  ", default)]
    pub last_price: Option<String>,
    #[serde(rename = "Close Price  ", default)]
    pub close_price: Option<String>,
    #[serde(rename = "Average Price ", default)]
    pub average_price: Option<String>,
    #[serde(rename = "Total Traded Quantity  ", default)]
    pub total_traded_quantity: Option<String>,
    #[serde(rename = "Turnover ₹  ", default)]
    pub turnover: Option<String>,
    #[serde(rename = "No. of Trades  ", default)]
    pub no_of_trades: Option<String>,
}
