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
    #[serde(rename = "PREV_CLOSE", deserialize_with = "deserialize_optional_f64")]
    pub prev_close: Option<f64>,
    #[serde(rename = "OPEN_PRICE", deserialize_with = "deserialize_optional_f64")]
    pub open_price: Option<f64>,
    #[serde(rename = "HIGH_PRICE", deserialize_with = "deserialize_optional_f64")]
    pub high_price: Option<f64>,
    #[serde(rename = "LOW_PRICE", deserialize_with = "deserialize_optional_f64")]
    pub low_price: Option<f64>,
    #[serde(rename = "LAST_PRICE", deserialize_with = "deserialize_optional_f64")]
    pub last_price: Option<f64>,
    #[serde(rename = "CLOSE_PRICE", deserialize_with = "deserialize_optional_f64")]
    pub close_price: Option<f64>,
    #[serde(rename = "AVG_PRICE", deserialize_with = "deserialize_optional_f64")]
    pub average_price: Option<f64>,
    #[serde(rename = "TTL_TRD_QNTY", deserialize_with = "deserialize_optional_f64")]
    pub total_traded_quantity: Option<f64>,
    #[serde(rename = "TURNOVER_LACS", deserialize_with = "deserialize_optional_f64")]
    pub turnover: Option<f64>,
    #[serde(rename = "NO_OF_TRADES", deserialize_with = "deserialize_optional_f64")]
    pub no_of_trades: Option<f64>,
}

fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            let clean = s.replace(",", "").trim().to_string();
            if clean.is_empty() || clean == "-" {
                Ok(None)
            } else {
                clean.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        None => Ok(None),
    }
}
