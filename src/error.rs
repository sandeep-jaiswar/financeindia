use pyo3::exceptions::{PyConnectionError, PyOSError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use thiserror::Error;

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
}

impl From<PyErr> for FinanceError {
    fn from(err: PyErr) -> Self {
        FinanceError::PyErr(err)
    }
}

impl From<FinanceError> for PyErr {
    fn from(err: FinanceError) -> PyErr {
        match err {
            FinanceError::Http(e) => PyConnectionError::new_err(e.to_string()),
            FinanceError::Json(e) => PyValueError::new_err(e.to_string()),
            FinanceError::Csv(e) => PyValueError::new_err(e.to_string()),
            FinanceError::Io(e) => PyOSError::new_err(e.to_string()),
            FinanceError::Zip(e) => PyRuntimeError::new_err(e.to_string()),
            FinanceError::Xml(e) => PyValueError::new_err(e.to_string()),
            FinanceError::UrlParse(e) => PyValueError::new_err(e.to_string()),
            FinanceError::Py(s) => PyRuntimeError::new_err(s),
            FinanceError::PyErr(e) => e,
            FinanceError::Runtime(s) => PyRuntimeError::new_err(s),
        }
    }
}

pub type FinanceResult<T> = Result<T, FinanceError>;
