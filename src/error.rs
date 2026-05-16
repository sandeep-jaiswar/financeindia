use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use thiserror::Error;

// Create Python exception classes
pyo3::create_exception!(financeindia, FinanceException, PyRuntimeError);
pyo3::create_exception!(financeindia, HTTPError, FinanceException);
pyo3::create_exception!(financeindia, ConnectionError, HTTPError);
pyo3::create_exception!(financeindia, TimeoutError, HTTPError);
pyo3::create_exception!(financeindia, StatusCodeError, HTTPError);
pyo3::create_exception!(financeindia, RateLimitError, HTTPError);
pyo3::create_exception!(financeindia, DataError, FinanceException);
pyo3::create_exception!(financeindia, JSONParseError, DataError);
pyo3::create_exception!(financeindia, CSVParseError, DataError);
pyo3::create_exception!(financeindia, XMLParseError, DataError);
pyo3::create_exception!(financeindia, ValidationError, FinanceException);
pyo3::create_exception!(financeindia, NetworkError, FinanceException);
pyo3::create_exception!(financeindia, UnknownError, FinanceException);

#[derive(Error, Debug)]
pub enum FinanceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Python callback error: {0}")]
    Py(String),
    #[error("Python error: {0}")]
    PyErr(PyErr),
    #[error("Runtime error: {0}")]
    Runtime(String),

    // New structured errors
    #[error("Rate limited: retry after {0:?} seconds")]
    RateLimited(Option<u64>),
    #[error("Invalid input: {0}")]
    Validation(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("HTTP status error: {0}")]
    StatusCode(u16, String),
}

impl From<PyErr> for FinanceError {
    fn from(err: PyErr) -> Self {
        FinanceError::PyErr(err)
    }
}

impl From<FinanceError> for PyErr {
    fn from(err: FinanceError) -> PyErr {
        match err {
            FinanceError::Http(e) => {
                if e.is_timeout() {
                    TimeoutError::new_err(e.to_string())
                } else if e.is_connect() {
                    ConnectionError::new_err(e.to_string())
                } else {
                    HTTPError::new_err(e.to_string())
                }
            }
            FinanceError::Json(e) => JSONParseError::new_err(e.to_string()),
            FinanceError::Csv(e) => CSVParseError::new_err(e.to_string()),
            FinanceError::Io(e) => NetworkError::new_err(format!("IO error: {}", e)),
            FinanceError::Zip(e) => DataError::new_err(format!("Zip error: {}", e)),
            FinanceError::Xml(e) => XMLParseError::new_err(e.to_string()),
            FinanceError::UrlParse(e) => ValidationError::new_err(format!("URL parse error: {}", e)),
            FinanceError::Py(s) => FinanceException::new_err(s),
            FinanceError::PyErr(e) => e,
            FinanceError::Runtime(s) => FinanceException::new_err(s),
            FinanceError::RateLimited(retry_after) => {
                let msg = match retry_after {
                    Some(secs) => format!("Rate limited: retry after {} seconds", secs),
                    None => "Rate limited: retry later".to_string(),
                };
                RateLimitError::new_err(msg)
            }
            FinanceError::Validation(msg) => ValidationError::new_err(msg),
            FinanceError::Network(msg) => NetworkError::new_err(msg),
            FinanceError::StatusCode(code, msg) => {
                StatusCodeError::new_err(format!("HTTP {}: {}", code, msg))
            }
        }
    }
}

pub type FinanceResult<T> = Result<T, FinanceError>;

// Helper to create rate limit errors from HTTP responses
pub fn rate_limited_error(retry_after: Option<u64>) -> FinanceError {
    FinanceError::RateLimited(retry_after)
}

// Helper to create validation errors
pub fn validation_error(msg: impl Into<String>) -> FinanceError {
    FinanceError::Validation(msg.into())
}

// Helper to create network errors
pub fn network_error(msg: impl Into<String>) -> FinanceError {
    FinanceError::Network(msg.into())
}

// Helper to create status code errors
pub fn status_code_error(status: u16, message: impl Into<String>) -> FinanceError {
    FinanceError::StatusCode(status, message.into())
}